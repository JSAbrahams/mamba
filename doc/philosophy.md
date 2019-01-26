# Philosophy of the language

An informal document that outlines the philosophy of the language for this language. Most of these topics are covered in
more detail in the language specification. This is just a general explanation of the features of language, how they fit
within the philosophy of the language and where they originated from.

> A language should make it easy to write clear descriptive unambiguous code, and difficult to write code that isn't 
those things

A key thing to keep in mind, especially when writing code in teams (which is how development software is usually 
written), is the following:

> Code is more often read than it is written

### Pragmatism over Ideology

### Readability, Keywords, and Syntax Sugar

### Null Safety and Error Handling

### Mutability and Immutability

### Design by Contract

### The Mathematical Roots of Computer Science

## General inspirations of the language

Language design is difficult. There is this saying in computer science, or engineering in general:

> A tool is either not used, or complained about

And this is just as true for programming languages. Often, languages are used in ways they were never intended. Often I
head people complain that a language is not suited for a certain task. And that is perfectly fine, it may be that it was
never designed for that problem domain. 

A language, in its simplest form, is meant to allow engineers, or people in general, to efficiently solve problems in a
certain domain. 

It is difficult to say whether a language is bad or not. I will say that certain features of certain languages, no
matter how well intentioned they may have been, or insignificant they seemed at the time, may not have been the best
choice in hindsight. But everything always seems so clear in hindsight. 

Over time, I think that we get better at language designs. Things that are often complained about can be weeded out, 
common programming mistakes can be discouraged to language design, and common patterns can be taken into account when
designing new languages.

No language is perfect, but there certainly is always room for improvement.

### Languages

The following is a list of programming languages that inspired this one in one way or another. This can either be
certain constructs or keywords in the language, or the philosophy of the language as a whole.

Language | Description | Inspired
---------|-------------|------------
Python   |  | Flexibility. Co-existence of functions and methods, or co-existence of functional and oop paradigms, and large portion of syntax
Java     |  |
C#       |  | 
Scala    |  | Everything is an object, including primitives. 
Kotlin   |  | Ranges. Type aliases, relying in keywords to define common use cases instead of having to write everything explicitly
Ada      |  | Custom data types (or type aliases) with ranges. Strict typing rules. Natural language over symbols, such using `and` stead of `&&`
C++      |  |
C        |  |
Eiffel   |  | Design by contract features. These include the `require` and `ensure` keywords, and the `retry` keywords
Haskell  |  | Pattern Matching features. Lack of mutability. Closeness of mapping with mathematical notation, for instance set constructor notation
Rust     |  | Error handling mechanisms. Strict rules regarding mutability
Ruby     |  | Syntax sugar, postfix `if` operator. Philosophy that flexibility is not inherently a bad thing
Swift    |  | Error handling mechanisms
Go       |  | Error handling mechanisms
MATLAB   |  | 
SmallTalk|  |
