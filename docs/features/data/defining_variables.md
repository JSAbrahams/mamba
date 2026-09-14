⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.2 📝 Data](README.md)

# 2.2.2 Defining Variables and Functions (or Methods)

Every definition must be preceded with the `def` keyword.
This is not necessary when reassigning however.

Functions and methods cannot be reassigned, values marked `mut` can, however.
A value is an expression which may be evaluated.

## Variables

A variable definition has the following structure:

    def [ mut ] <identifier> [ : <type> ] := <expression>

For instance, a variable `x` is assigned to as such:

    def x := <expression>

Or:

    def mut x := <expression>

Use this if `x` has to be mutable.
A definition is immutable unless we mark it `mut`, so we reassign the second `x` but not the first:

    x := <expression>

The same marker works on a function argument, and on the `self` argument of a method.
A method that assigns to its own fields must take `mut self`.

## Functions

A function definition has the following structure:

    def <identifier> ( { <identifier> [ : <type> ] [ := <expression> ] } ) [ -> <type> ] [ ! ( <type> | "{" <type> { "," <type> } "}" ) ] := <expression or statement>

So for instance, we can define a function as follows:

    def factorial(n: Int) -> Int :=
        if n = 0 then 1
        else n * factorial(n - 1)

A few things to note:

- The function is named `factorial`
- It takes an argument `n`, which is a `Int`.
  As such, we write `n: Int`
- The function returns an integer, which is why we write `-> Int` before proceeding to the body of the function
- The body of a function follows after the `:=`.
  The body of a function can either be an expression or a statement.
  If it is more than one, it is a block, written between `do` and `end`.

We must always include the types of the argument of a function.
We may however omit the return type of a function if it is inferrable from the body.
The return type can also be omitted if the function does not return anything.
This is effectively the same as saying the function returns `None`.

The optional `!` after the return type lists the errors the function may raise.
See [Error Handling](../safety/error_handling.md).

### Default values

We can have default values:

    class MyClass where
        def mut my_field: Int := 5
        def my_method(mut self, x: Int, y: Int := 2) := self.my_field := x + y
    end

We can now call the method as such:

    def mut my_class := MyClass()
    my_class.my_method(10, 2)

Or, leaving `y` to its default:

    my_class.my_method(10)

Note that a method takes an explicit `self` argument, and that its fields are reached through it.
So it is `self.my_field`, and not a bare `my_field`.
