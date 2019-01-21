# Defining Variables and Functions (or Methods)

Every definition must be preceded with the `def` keyword. This is not necessary when reassigning however.

Functions and methods cannot be reassigned, mutable values can however. A value is an expression which may be evaluated.

A variable `x` is assigned to as such:\
`def x <- <expression>` or `def mut x <- <expression>`

A function (or method) `f` is assigned to as such:\
`def f (x : Int) <- x + 1` or `def f (x : Int): Int <- x + 1`\
The return type of the function may be omitted if it can be inferred.

A function may also be anonymous:\
`(x) <- x + 1`\
In this case the types of the function  arguments may be omitted, provided that these can be inferred elsewhere.
