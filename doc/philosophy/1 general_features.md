# General Features

Here I outline in short some of the core features of the language, and their motivation. These are covered in greater 
detail in their respective chapters. Those however serve more as a solicitation and not so much as a motivation for
their existence. There are multiple programming paradigms in computer science.

Paradigm                     | Description
-----------------------------|-------------
Procedural Programming       | Use a set of functions, or procedures, to carry out computations
Object Oriented Programming  | Model everything as an object. They may carry data, but on a conceptual level, they define behaviour
Functional Programming       | Treat computation as a mathematical function, there is no concept of state, and there are no side-effects

Each come with their own (or overlapping) set of philosophies and schools of thought. Certain languages stick to a
single paradigm. SmallTalk to Object Oriented Programming, Haskell to functional programming, Java historically to 
Object Oriented, though functional languages have been added retroactively, for better or worse.

## Readability, Keywords, and Syntax Sugar

One thing that can significantly hinder the readability of source code is syntax noise. One way to combat this is
through use of syntax sugar. Another however is to use clear keywords, and, in my opinion, less of a reliance on
symbols:

* Use `and` instead of `&&`, i.e. `alice.is_online and bob.is_online` instead of `alice.is_online && bob.is_online`
* Use `or` instead of `||`, i.e.  `foo() or bar()` instead of `foo() || bar()`
* Use `not` instead of `!`, i.e. `if not productive then drink_coffee()` instead of `if !productive then drink_coffee()`
* Use indentation to denote code blocks instead of `{` and `}`

There are a few other examples that are not listed here. The idea however in general is that the language maps more 
closely to the naturally spoken English language:

    foreach composer in composers
        if composer.death is undefined then
            println "[composer] has not died."
        else 
            def years_ago -> today - composer.death
            pintln "[composer] died [years_ago] years ago"
            
Without knowing the language, when reading above, it should be relatively clear that:

* We are iterating over a set (or collection) of composers
* We are checking whether a composer has died
* If a composer has died, we print how long ago that was. Presumably today is a date

Notice how little program specific syntax there is:
 
* We use `[` `]` to insert variables into strings
* We use `:` to denote what type `ago` is
* `if` and `else` are used for program flow, and `println` to print something to the screen
* Indentation is used to denote code blocks, making it easy for the eyes to follow what is being done where and when

## Null Safety and Error Handling

The null value, a value which is meant to symbolize the concept of nothing, has been the bane of many a programmer, with
headlines such as:

> Null Pointers, the billion (or perhaps trillion) dollar mistake

Null safety is an oft raised topic in computer science. Null values can either break the application flow of an 
application by throwing an exception (such as in Java), or simply result in undefined behaviour (such as in C++).

## Mutability and Immutability

Immutability, which allows us to change a value of a variable, brings with it great flexibility, but in certain
situations this flexibility comes at the detriment of safety.

When a variable is immutable, it should truly be immutable. Often I see that even when a variable is declared immutable,
it is possible to modify it's internal fields in object oriented programming languages. This led to one of the core
features of the language.

### Mutability Propagation

Mutability propagation basically propagates the immutability of a variable to its contents, so that when it is declared
to be immutable, it's fields are in essence locked, and cannot be changed.

## Type Safety

In programming, there is often a distinction made between static and dynamic typing. As stated before however, we want
to only have checked Exceptions in our application. Once we run an application, we expect its behaviour to reflect what
was written.

If the application were dynamically typed, we would constantly have to verify that variables are indeed what they claim
to be. I have a variable `beethoven`, but is that an instance of `Composer` or `Person`? How do I check this, what
methods can I use?
   
    # how can I be sure that this function argument is a composer?
    def my_function (composer) -> composer.composer_method()
    
To this end, we use types. A user defines a class `Composer`, which defines the behaviour of a composer:

    class Composer
    
    def composer_method(): Int
    
And then we define the `my_function` as such:

    def my_function (composer: Composer): Int -> composer.composer_method()
    
Now, in the body of the function, we can rest easy knowing that the passed variable is indeed a composer. It is actually
now impossible to pass another variable type to the function, as this is checked by the type checker, which will give an
error, meaning that the program wil not run.

In some programming languages, we have to explicitly state the type of each variable. This however makes the application
rather verbose. Take:

    def Int x <- 10                     # x is obviously an integer
    def Complex c <- Complex(10, 20)    # from the right hand side it is already clear that c is complex
    # and so forth    

Instead, we use type inference. Every variable is still assigned a type, which is used by the type checker, but the type
itself is inferred from an expression

    def x <- 10                 # x has type Int, we know this because 10 is an Int
    def c <- Complex(10, 20)    # c has type Complex
    def y <- 20.1               # 20.1 uses decimal notation, so we know y is a real number, or Real
    
    def z: Real <- 10.5         # In some situations however, you still might want to explicitly mention the type

> In general, something is clear, or inferable, from context, don't make the developer write it out in full

### Type Aliases and Type Refinement

We can also use type aliases and type refinement to further refine types by adding conditions to types:

    type DeadComposer <- Composer where
        self.death isnt undefined else Err("Composer is not dead.")

We now rewrite my_function so it only works for `DeadComposer`s:

    def my_function (composer: DeadComposer): Int <- today.year - composer.death.year
    
Again, we can rest assured that `composer` is a `DeadComposer` in the body of the function. To use such a function, we 
must explicitly cast a composer:

    def chopin <- Composers("Chopin")
    
    if chopin isa DeadComposer
        def years_ago <- my_function(chopin)                    # chopin is dynamically casted to a DeadComposer
        println ("[chopin.name] died [years_ago] years ago.")

This draws on concepts of "Design by Contract", which is elaborated on below.

## Design by Contract

Design by Contract is a software correctness methodology. It defines a set of preconditions that must be satisfied for a
function to perform its operations, and a set of post-conditions that must be adhered to after the function has
completed its operation. In effect, is verifies the program is in a valid state for the function to be executed, and it
verifies that the program is in one of the expected state after execution of the function.

This term was conceived by Bertrand Meyer, and implemented in the Eiffel language. 

### Type Refinement

Type refinement can also be used to enforce design by contract. Design by contract is not only used to check the
arguments of a method, but also to check that the object is in a correct state. Usually, the state of an object is left
implicit. By using type aliases with accompanying checks, we can explicitly name the different states an object can be
in.

For instance, take a simple server object, which forms a connection with a server. Then this can either be connected or
disconnected (for the sake of simplicity). We can then define two states:

    type connectedServer <- Server where
        self connected else Err("Not connected.")
        
    type disconnectedServer <- Server where
        self not connected else Err("connected.")

Of course, this is a rather trivial example, but it can be more useful when states become more complicated. It can also
be used as the return type of a method to signify that a object in is that state after execution of that method.

## The Mathematical Roots of Computer Science

Computer Science has its roots in mathematics. I wanted to reflect this in the language. An obvious example is the
set builder notation:

    def programmers <- { x | x in People, x.profession is PROGRAMMER }
