⬅ [🏠 Home](../README.md)

⬅ [1 💭 Philsophy of the Language](README.md)

# 1.2 Readability

## Syntax and Syntax Sugar

One thing that can significantly hinder the readability of source code is syntax noise. 
Therefore, I tried to limit the amount of symbols and keywords in the language.
I believe that when reading a piece of code, one should be able to understand what is happening without being hindered by excessive syntax.

One way to make code "look" more elegant, and ideally more readable, is through use of syntax sugar.
This allows developers to, ideally, express their ideas in a more elegant manner. 
Syntax sugar however comes with its own set of problems, one of which is that it can introduce a new way of solving the same problem.
We therefore aim to use syntax sugar sparingly, but still often enough so the language does not become overly verbose.
A few examples are shown below:

- Functions and methods with no arguments, or a single argument, can be called without brackets.
- In a `foreach`, if we iterate over a collection of tuples, we can omit the brackets (just as in Python).
- We can define a range using `..` and `..=`.
- The negation of equality `=` is `/=`, instead of having to negate the entire equality using `not`.

As stated before, we also decrease the reliance on symbols, which may be another source of syntax noise.
This should make the language more closely map to the english language, but not so close as to introduce ambiguity into the language.
A few examples are given below:

- Use `and` instead of `&&`, i.e. `alice.is_online and bob.is_online` instead of `alice.is_online && bob.is_online`.
  We can even write `alice is_online and bob is_online`.
- Use `or` instead of `||`, i.e.  `foo() or bar()` instead of `foo() || bar()`.
- Use `not` instead of `!`, which can easily be missed when read. 
  So we write `if not productive then drink_coffee()` instead of `if !productive then drink_coffee()`.
- Use indentation to denote code blocks instead of `{` and `}`, just as in Python.

Mamba also uses arrow notation to more clearly denote the flow of data:
- `<-` is used to assign to variables. 
  It denotes data flowing from the expression on the right to the identifier on the left.
- `->` is used within type definitions of functions and methods.
- `=>` is used to denote the control flow of the application.
  This is used for two reasons:
  - It more clearly differentiates data manipulation from application control flow.
  - It avoid ambiguity in the grammar by differentiating anonymous functions from control flow in certain situations.

This, combined with (sparing) use of syntax sugar, should ideally make the language easier to read.
Take for instance the following piece of code:

    foreach composer in composers do
        if composer.death is undefined then
            print "[composer] has not died."
        else 
            def years_ago <- today - composer.death
            print "[composer] died [years_ago] years ago."

Without any knowledge of Mamba, the reader should ideally, when reading above, be able to deduce that:

- We are iterating over a set (or collection) of composers.
- We are checking whether a composer has died.
- If a composer has died, we print how long ago that was. 
  Presumably today is a date.

Notice how little program specific syntax there is:
 
- We use `[` ... `]` to insert variables into strings.
- `if` and `else` are used for program flow, and `print` to print something to the screen.
  - Note how we use the postfix notation when calling `print`, so we don't need to wrap the whole string in brackets.
- Indentation is used to denote code blocks, making it easy for the eyes to follow what is being done where and when.
