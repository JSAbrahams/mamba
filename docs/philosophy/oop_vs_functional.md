⬅ [🏠 Home](../README.md)

⬅ [1 💭 Philsophy of the Language](README.md)

# 1.3 OOP and Imperative Versus Functional, or, Idealism versus Pragmatism

There are multiple programming paradigms in computer science.
A few of the popular one's are outlined in this short, non-exhaustive list:

Paradigm                     | Description
-----------------------------|-------------
Procedural Programming       | Use a set of functions, or procedures, to carry out computations.
Object Oriented Programming  | Model everything as an object. They may carry data, but on a conceptual level, they define behaviour.
Functional Programming       | Treat computation as a mathematical function, with no side-effects.

Each come with their own (or overlapping) set of philosophies and schools of thought. 
Certain languages stick to a single paradigm. 
SmallTalk, for instance, sticks to object oriented programming, and Haskell to functional programming.
Java has stuck mostly to object oriented, though it does make an exception for primitives, and has retroactively added features found in functional languages such as anonymous functions.

Mamba aims to provide a mix of the object oriented and functional programming paradigms.
It aims to improve upon Python by introducing static type checking, null safety, and a bunch of other features, such as type refinement.
We also use features from other languages which have proven successful over time.

## State and Statelessness

One thing that functional programmers often state to be a major benefit of functional programming is that there is no state.
This allows us to guarantee the following: `for any f, x, y: if x = y, then f x = f y`
