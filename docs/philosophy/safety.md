⬅ [🏠 Home](../README.md)

⬅ [1 💭 Philsophy of the Language](README.md)

# 1.5 Safety

## Null Safety

The `None` or `null` value, or `undefined` in cases of Mamba, is value which is meant to symbolize the concept of nothing.
It can be useful in some cases.
Implemented poorly in a language however can be the source of many bugs.
It has lead to headlines akin to:

> Null Pointers, the billion dollar mistake

Null values can either break the application flow of an application by throwing an exception, such as in Java, or simply result in undefined behaviour, such as in C++.
A language with no null safety basically nullifies much of the functionality of a type system.
If I write a function, and I say that it returns an integer, a user of said function should not have to worry about the return value potentially being `null` (or `None` or `undefined`).
Over the years, multiple languages have tried to implement null safety, with the two prevailing strategies being as follows:

- The language does not contain `null` as a concept, but has a monad.
  This often called an `Optional`, and can either be `None`, or `Some`, with `Some` wrapping the return value of say a function.
- Null safety is baked into the language, and is enforced by the type system itself.
  This is for instance the approach that Kotlin uses.

For Mamba I opted to bake null safety into the language itself.
`?` is appended to types that may be `undefined`. 
If this has not been appended, say in the signature of a method, this method may never return `undefined`.
If the type checker can not ensure that this is the case (or it detects that this is indeed the case), then it will raise an error and the code will not compile.
`?.` can be used to call a method of a expressions that may be null, and it means a method is only called if the expression is not `undefined`.

```
# g never returns undefined.
# If we would try this we would get a type error.
def g(x: Int): Int => if x > 2 then x + 2 else x - 2

# f may return undefined
def f(x: Int): Int? => if x > 2 then x * 7 else undefined

def x <- f(2)?.is_even() # is_even is called, and we get true
def y <- f(3)?.is_even() # f(3) is undefined, so is_even() is never called and y is now also undefined

# note that both x and y have type Bool?, meaning that they may both be undefined.
```

As with most language, Mamba also uses bounds checking.
Unlike Python however, we don't wrap the value around if we access outside the bounds of an array, but we raise an error, which should be handled.
This should ideally make it easier to track down bugs, which might otherwise be undetected for some time.
This adheres somewhat to the **fail fast** philosophy.
