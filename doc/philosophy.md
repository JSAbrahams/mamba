# Philosophy of the language

Whilst designing the language, I made the following rule for myself:

> A language should make it easy to write clear descriptive unambiguous code, and difficult to write code that isn't 
those things

I also tried to keep the following in mind, which often bears repeating:

> Code is more often read than it is written

As such, I wanted to make a language that is both easy to write in, but also easy to read. There's also this famous
thing in Computer Science called the "no free lunch theorem", and that holds true for language design as well. In short,
it means that for every decision there are trade-offs, every advantage comes with its own baggage of problems.

* Increased flexibility might make it easier to write bug prone code
* Increased flexibility might come at the cost of performance
* A strongly typed language might be more cumbersome to work with
* ...

And the list just keeps on growing. Language design is tricky to say the least. There is this saying in computer 
science, or engineering in general:

> A tool is either not used, or complained about

And this is just as true for programming languages. I often see online blogs or posts talking about what the "best"
language is. I language, ideally, is designed to solve problems in a certain domain reasonably well. Admittedly, what 
this domain is becomes harder to define for more general purpose languages. 

It is difficult to say whether a language is bad or not. I will say that certain features of certain languages, no
matter how well intentioned they may have been, or insignificant they seemed at the time, may not have been the best
choice in hindsight. But hindsight is 20/20 as they say, and I don't think there is much point in seriously arguing 
about what language is best.

Over time, I think that we have gotten better at designing languages. Things that are often complained about can be 
weeded out, common programming mistakes can be discouraged via language design, and common patterns can be taken into 
account when designing new languages, so these can be expressed in a more idiomatic manner. But as stated before, what
a common pattern is might heavily depend on the domain of problems being solved.

No language is perfect, but there certainly is always room for improvement. Below I outline some of the design decisions
made, why these were made, and what they aim to achieve (and/or improve).

- Joël Abrahams, 2019

### Pragmatism over Ideology

There are multiple programming paradigms in computer science, with the most well known being:

Pradigm                      | Description
-----------------------------|-------------
Procedural Programming       | 
Object Oriented Programming  | 
Functional Programming       |

Each come with their own (or overlapping) set of philosophies and schools of thought. Certain languages stick to a
single paradigm. SmallTalk to Object Oriented Programming, Haskell to functional programming, Java historically to 
Object Oriented, though functional languages have been added retroactively, for better or worse.

### Readability, Keywords, and Syntax Sugar

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
            def years_ago <- today - composer.death
            pintln "[composer] died [years_ago] years ago"
            
Without knowing the language, when reading above, it should be relatively clear that:

* We are iterating over a set (or collection) of composers
* We are checking whether a composer has died
* If a composer has died, we print how long ago that was. Presumably today is a date

Notice how little program specific syntax there is:
 
* We use `[` `]` to insert variables into strings
* We use `:` to denote what type `ago` is
* `if` `else` is used for program flow, and `println` to print something to the screen
* Indentation is used to denote code blocks, making it easy for the eyes to follow what is being done where and when

### Null Safety and Error Handling

> Null Pointers, the billion (or perhaps trillion) dollar mistake.

Null safety is an oft raised topic in computer science. They either break the application flow of an application by
throwing an exception (such as in Java), or simply result in undefined behaviour (such as in C++).

### Mutability and Immutability

Immutability, which allows us to change a value of a variable, brings with it great flexibility, but in certain
situations this flexibility comes at the detriment of safety.

### Type Safety

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
    def y <- 20.1               # 20.1 uses decimal notation, so we know y is a real number
    
    def z: Real <- 10.5         # In some situations however you might stil want to explicitly mention the type

We can also use type aliases and type refinement to further refine types by adding conditions to types:

    type DeadComposer <- Composer where
        self.death isnt undefined else Err("Composer is not dead.")

We now rewrite my_function so it only works for `DeadComposer`s:

    def my_function (composer: DeadComposer): Int -> today.year - composer.death.year
    
Again, we can rest assured that `composer` is a `DeadComposer` in the body of the function. Now we can be sure that the
variable is a `Composer` contains a defined `death` field. To use such a function, we must explicitly cast a composer:

    def chopin <- Composers("Chopin")
    
    if chopin isa DeadComposer
        def years_ago <- my_function(chopin)                    # chopin is dynamically casted to a DeadComposer
        println ("[chopin.name] died [years_ago] years ago.")

This draws on concepts of "Design by Contract", which is elaborated on below.

### Design by Contract

Design by Contract is a software correctness methodology. It defines a set of preconditions that must be satisfied for a
function to perform its operations, and a set of post-conditions that must be adhered to after the function has
completed its operation. In effect, is verifies the program is in a valid state for the function to be executed, and it
verifies that the program is in one of the expected state after execution of the function.

This term was invented by Bertrand Meyer, and implemented in the Eiffel language. 

### The Mathematical Roots of Computer Science

Computer Science has its roots in mathematics. I wanted to reflect this in the language.

## General inspirations of the language

The following is a list of programming languages that inspired this one in one way or another. This can either be
certain constructs or keywords in the language, or the philosophy of the language as a whole.

Language  | Description | Inspired
----------|-------------|------------
Python    |  | Flexibility. Co-existence of functions and methods, or co-existence of functional and oop paradigms, and large portion of syntax
Java      |  | OOP concepts
C#        |  | OOP concepts
Scala     |  | Everything is an object, including primitives. 
Kotlin    |  | Ranges. Type aliases, relying in keywords to define common use cases instead of having to write everything explicitly
Ada       |  | Custom data types (or type aliases) with ranges. Strict typing rules. Natural language over symbols, such using `and` stead of `&&`
C++       |  | OOP concept
C         |  | 
Eiffel    |  | Design by contract features, the `retry` keyword
Haskell   |  | Pattern Matching features. Lack of mutability. Closeness of mapping with mathematical notation, for instance set constructor notation
Rust      |  | Error handling mechanisms. Strict rules regarding mutability
Ruby      |  | Syntax sugar, postfix `if` operator. Philosophy that flexibility is not inherently a bad thing
Swift     |  | Error handling mechanisms
Go        |  | Error handling mechanisms
MATLAB    |  | Concepts of flexibility
SmallTalk |  | OOP concepts, emphasis on program state
JavaScript|  | Interchangeability of variables and functions 

