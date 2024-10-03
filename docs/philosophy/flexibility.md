⬅ [🏠 Home](../README.md)

⬅ [1 💭 Philsophy of the Language](README.md)

# 1.4 Flexibility

In addition to readability of source code, a language should provide a nice balance between rigorousness and flexibility.
A language should be flexible enough for developers to express their ideas in a clear precise manner.
Ideally, a developer should not have to jump through loops to write up their ideas in code.
A language should however not introduce too much ambiguity, which for instance can happen when there is too much syntax sugar present, allowing for multiple ways of doing the same (trivial) thing.

Truth be told, flexibility is more of an illusion when it comes to programming languages.
No matter what way you spin it, a language is a collection of strictly defined grammar and semantic rules.
Computers operate using well-defined instructions, which by their nature are unambigious.
A programming language serves as a bridge between these instructions and human speech.
It is not quite as natural (and thankfully not as ambiguous) as human speech, but it is not quite as rigorous (and unreadable) as machine instructions.

Where a language lies on this spectrum varies greatly however.
Loosely speaking, languages closer to human speech are often regarded as "high level" languages, whereas languages closer to machine level instructions are often regarded as "low level" langauges.

## Challenging the Python Philosophy

As Mamba resides in the Python ecosystem, I feel that it is important to motivate its existence.
While I by no means dislike Python, and have much respect for the language, I feel as though there are certain areas of the language which could be improved upon.
Here I aim to explain and motivate these so called "improvements".

Python is definitely a high-level language.
As a Python programmer, you needn't worry about pointers, memory leaks, the heap and stack, and so on.

### Types

As a Python programmer, however, you also needn't worry about stating the type of something, making it quite a flexibile language.
This is something that Mamba challenges, as Mamba does require that everything has a clearly defined type.
This, ideally, should make it more difficult to write buggy code.
For intance, if a function expects and integer, and gets a string, Mamba will catch this at compile time, whereas Python requires you to actually run the function.

In addition to this, in my opinion, it is far easier to reason about the correctness of an application if we clearly state what the type of each expression is.
This is also why Mamba has type refinement features, as we can assign certain conditions to types to make it easier to reason about our application.
For instance, we have the type `Int` baked into the language, which means that said expression is an integer.
We can state that a function `f` expects an argument `x`, which should be an `Int` (and not, say, a `String`).
However, we can go one step further.
We can define a type `PositiveInt`, which has as a property that, if `x` is a `PositiveInt`, that `x >= 0` is always true.
Then, if `f` has as domain all positive integers, we can say that argument `x` must be a `PositiveInt`, and we can be sure that in the body of said function `x` will always be positive.

Mamba however does also use type inference so applications don't become overly verbose.
So we can write `def x := 10`  as opposed to writing `def x: Int := 10` (which is also correct).
Here it is obvious that `x` is an integer, so it is not required to state it explicitly.

### Null Safety

This closely ties with the above.
In Python, something may be undefined (`None`), and we may not be aware of this.

Mamba aims to introduce mechanisms which ideally should make developers aware of situations where something may be undefined.

### Error handling

Mamba also aims to introduce more explicit error handling mechanisms to the Python ecosystem.
While these may be overkill for small scripts, for which Python is often used, it can save a lot of time in the long run for larger applications.
More explicit handling or errors, much like null safety features, should ideally make the developer more aware of what might and might not go wrong in an application.

