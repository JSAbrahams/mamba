# Defining Variables and Functions (or Methods)

Every definition must be preceded with the `def` keyword. This is not necessary when reassigning however.

Functions and methods cannot be reassigned, mutable values can however. A value is an expression which may be evaluated.

## Variables

A variable `x` is assigned to as such:\
`def x <- <expression>` or `def mut x <- <expression>`

## Functions and Methods

A function (or method) `f` is assigned to as such:\
`def f (x : Int) -> x + 1` or `def f (x : Int): Int -> x + 1`\
The return type of the function may be omitted if it can be inferred. Notice that here we have `->` instead of `->`. The
arrows symbolize the flow of the application. When we assign to a variable, we first evaluate what is right of the arrow
before assigning it to the variable. In the case of functions however, we do not evaluate the right hand side. Instead,
we only evaluate it upon a function call, with the given variables which are then bound in the body of the function, 
which is on the right hand sight of the arrow. Therefore, upon calling a function, we first bind the variables on the
left hand side of the arrow before continuing on the right hand side of the arrow.

A function may also be anonymous:\
`(x) -> x + 1`\
In this case the types of the function  arguments may be omitted, provided that these can be inferred elsewhere.
