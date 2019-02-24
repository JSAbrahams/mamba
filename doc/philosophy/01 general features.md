# General Features

Here I outline in short some of the core features of Mamba, and their motivation, and to reveal a bit of the philosophy behind the language.
These are covered in greater detail in their respective chapters. 

There are multiple programming paradigms in computer science, as can be seen below in this non-exhaustive list:

Paradigm                     | Description
-----------------------------|-------------
Procedural Programming       | Use a set of functions, or procedures, to carry out computations.
Object Oriented Programming  | Model everything as an object. They may carry data, but on a conceptual level, they define behaviour.
Functional Programming       | Treat computation as a mathematical function, with no side-effects.

Each come with their own (or overlapping) set of philosophies and schools of thought. 
Certain languages stick to a single paradigm. 
SmallTalk for instance sticks to object oriented programming, and Haskell to functional programming.
Java has stuck mostly to object oriented, though it does make an exception for primitives, and has retroactively added features found in functional languages such as anonymous functions.

Mamba aims to provide a mix of the object oriented and functional programming paradigms.
Crudely speaking, it aims to improve upon Python by providing types and other features that make it easier to write bug-free code.
We also use features from other languages which have proven successful over time.

## Syntax, Syntax Sugar, and Readability

One thing that can significantly hinder the readability of source code is syntax noise. 
One way to combat this is through use of syntax sugar, which allows programmers to, ideally, express their ideas in a more elegant manner. 
Syntax sugar however comes with its own set of problems.
The same idea can be expressed in more than one way, increasing the cognitive load of reviewing code and learning the language.

We therefore aim to use syntax sugar sparingly, but still often enough so the language does not become too verbose.
A few examples are shown below:

* Functions and methods with no arguments, or a single argument, can be called without brackets.
* In a `foreach`, if we iterate over a collection of tuples, we can omit the brackets.
* We can define a range using `..` and `..=`.
* The negation of equality `eq` is `neq`, instead of having to negate the equality using `not`.

For the sake of readability, we also decrease the reliance of symbols, which may be another source of syntax noise, decreasing the readability of the language.
This should make the language more closely map to the english language, but not so close as to introduce ambiguity into the language.
A few examples are given below:

* Use `and` instead of `&&`, i.e. `alice.is_online and bob.is_online` instead of `alice.is_online && bob.is_online`.
  Or even `alice is_online and bob is_online`.
* Use `or` instead of `||`, i.e.  `foo() or bar()` instead of `foo() || bar()`.
* Use `not` instead of `!`, i.e. `if not productive then drink_coffee()` instead of `if !productive then drink_coffee()`.
* Use indentation to denote code blocks instead of `{` and `}`.

Mamba also uses arrow notation to more clearly denote the flow of data:
* `<-` is used to assign to variables. 
  It denotes data flowing from the expression on the right to the identifier on the left.
* `->` is used to denote (anonymous) functions.
  It denotes variables assigned to the arguments on the left flowing into the body of the function.
* `=>` is used to denote the control flow of the application.
  This is used for two reasons:
  * It more clearly differentiates data manipulation from application control flow.
  * It avoid ambiguity in the grammar by differentiating anonymous functions from control flow in certain situations.

This combined with (sparing) use of syntax sugar should ideally make the language easier to read.
Take for instance the following piece of code:

    foreach composer in composers
        if composer.death is undefined then
            println "[composer] has not died."
        else 
            def years_ago -> today - composer.death
            pintln "[composer] died [years_ago] years ago"
            
Without any knowledge of Mamba, the reader should ideally, when reading above, be able to deduce that:

* We are iterating over a set (or collection) of composers.
* We are checking whether a composer has died.
* If a composer has died, we print how long ago that was. 
  Presumably today is a date.

Notice how little program specific syntax there is:
 
* We use `[` `]` to insert variables into strings.
* We use `:` to denote what type `ago` is.
* `if` and `else` are used for program flow, and `println` to print something to the screen.
  * Note how we use the postfix notation when calling `println`, so we don't need to wrap the whole string in brackets.
* Indentation is used to denote code blocks, making it easy for the eyes to follow what is being done where and when.

On a final note, I'd like to say that I believe the syntax of Python to be one of the most elegant.
I once heard someone say:

> When writing pseudocode, I end up writing Python code

That also happens to be the original intent of Mamba, when I started creating the language.
I wanted to create a language that felt and looked like pseudocode.
Only during the second attempt when designing the language did I stumble upon the idea to closely map the language syntax to that of the python language.

In the following chapters however I do elaborate on some features that Mamba has that aim to improve upon the Python language.
Let it be said however that I do not think that Python is a bad language by any stretch of the imagination.
It is a language I hold much respect for, and one that I enjoy writing code in.

## Null Safety, and Error Handling

The `null` value, or the `undefined` value in cases of language such Javascript or Mamba, is value which is meant to symbolize the concept of nothing.
It can be useful in some cases. 
Implemented poorly in a language however it is often the root cause of the null pointer exception, error, or whatever terminology one wishes to use, the bane of many a programmer.
It has lead to headlines similar to the following:

> Null Pointers, the billion dollar mistake

Null values can either break the application flow of an application by throwing an exception, such as in Java, or simply result in undefined behaviour, such as in C++.
A language with no null safety basically nullifies much of the functionality of a type system.
If I write a method, and I say that it returns an integer, a user of said function should not have to worry about the potential of the return value potentially being `null` or undefined unless explicitly stated.
Over the years, multiple languages have tried to implement null safety, with the two prevailing strategies being as follows:

* The language does not contain `null` as a concept, but has a monad, often called an `Optional`, which can either be a value or nothing.
* `null` safety is baked into the language, and is enforced by the type system itself.
  This is for instance the approach that Kotlin uses.

Buffer overflows might however be the trillion dollar mistake.
As with most language, it did opt to use bounds checking.
However, unlike other scripting languages such as Ruby or Python, I opted to raise an error when we attempt to access outside the bounds of an array (or collection), much like languages such as Java.
This should ideally make it easier to track down bugs, which might otherwise be undetected for some time.
This adheres somewhat to the **fail fast** philosophy.

Handling errors is something that is difficult to do well in language.
As a developer, it can be difficult to determine where and when we should handle such errors.
Doing this elegantly, whilst at the same time remaining explicit, is also difficult.

## Mutability and Immutability

Immutability, which allows us to change a value of a variable, brings with it great flexibility, but in certain
situations this flexibility comes at the detriment of safety.

When a variable is immutable, it should truly be immutable. Often I see that even when a variable is declared immutable,
it is possible to modify it's internal fields in object oriented programming languages.

## Type Safety

In programming, there is often a distinction made between static and dynamic typing. 
As stated before however, we want to only have checked Exceptions in our application. 
Once we run an application, we expect its behaviour to reflect what was written.

If the application were dynamically typed, we would constantly have to verify that variables are indeed what they claim to be.
I have a variable `beethoven`, but is that an instance of `Composer` or `Person`? How do I check this, what methods can I use?
   
    # how can I be sure that this function argument is a composer?
    def my_function (composer) -> composer.composer_method()
    
To this end, we use types. A user defines a class `Composer`, which defines the behaviour of a composer:

    class Composer
        def composer_method(): Int
    
And then we define the `my_function` as such:

    def my_function (composer: Composer): Int -> composer.composer_method()
    
Now, in the body of the function, we can rest easy knowing that the passed variable is indeed a composer. 
It is actually now impossible to pass another variable type to the function, as this is checked by the type checker, which will give an error, meaning that the program wil not run.

In some programming languages, we have to explicitly state the type of each variable. 
This however makes the application rather verbose. 
Take for instance:

    def x: Int <- 10                     # x is obviously an integer
    def c: Complex <- Complex(10, 20)    # from the right hand side it is already clear that c is complex

Instead, we can use type inference. 
The type of every variable is inferred from the expression on the right hand side.

    def x <- 10                 # x has type Int, we know this because 10 is an Int
    def c <- Complex(10, 20)    # c has type Complex
    def y <- 20.1               # 20.1 uses decimal notation, so we know y is a real number, or Real
    
    def z: Real <- 10.5         # In some situations however, you still might want to explicitly mention the type

The program is still statically typed, but now we don't require the developer to write everything out in full.

I believe this to be a far superior system to dynamic typing, or duck typing.
Type inference, to me, seems to be a nice compromise between having a strong static type system and low verbosity of a language, say Python.

### Type Aliases and Type Refinement

We can also use type aliases and type refinement to further refine types by adding conditions to types:

    type DeadComposer <- Composer where
        self.death isnt undefined else Err("Composer is not dead.")

We now rewrite my_function so it only works for `DeadComposer`s:

    def my_function (composer: DeadComposer): Int <- today.year - composer.death.year
    
Again, we can rest assured that `composer` is a `DeadComposer` in the body of the function. 
To use such a function, we must explicitly cast a `Composer`:

    def chopin <- Composers("Chopin")
    
    if chopin isa DeadComposer
        def years_ago <- my_function(chopin)                    # chopin is dynamically casted to a DeadComposer
        println ("[chopin.name] died [years_ago] years ago.")

This draws on concepts of **Design by Contract** philosophy.

Furthermore, it also allows us to explicitly define the state of an object, something which is often left ambiguous.
For instance, we can say a server is connected or disconnected by doing the following:

    type Server
        def private connected: Boolean
        def send_message(self: ConnectedServer, message: String): String

    type ConnectedServer isa Server where
        connected else Err("Server is not connected")
        
And we may then elsewhere implement this `Server` interface:

    class MyServer isa Server
        def private connected <- false
        
        def init() -> ...
        
        def connect(ip: IpAddress) -> ...
        
        # You can only call this function if I am a connected server
        def send_message(self: ConnectedServer, message: String) -> ...

This is a rather trivial example, but it shows how we can explicitly name the state of server.
What a state entails is centrally defined, and ideally part of the interface, instead of spread of the entire code base, and it can also to some extent be checked by the type checker.
