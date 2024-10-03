⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.2 📝 Data](README.md)

# 2.2.2 Defining Variables and Functions (or Methods)

Every definition must be preceded with the `def` keyword. 
This is not necessary when reassigning however.

Functions and methods cannot be reassigned, mutable values can however. 
A value is an expression which may be evaluated.

## Variables

A variable definition has the following structure:
    
    def [ mut ] <string> := <expression>

For instance, a variable `x` is assigned to as such:

    def x := <expression>

Or: 

    def fin x := <expression>

If `x` has to be immutable.

## Functions

A function definition has the following structure:

    def <string> ( { <expression> [ : <expression> ] } ) [ : <expression> ] => <expression or statement>

So for instance, we can define a function as follows:

    def factorial(n: Int): Int =>
        if   n = 0 then 1
        else n * self.factorial (n - 1) 

A few things to note:
-   The function is named `factorial`
-   It takes an argument `n`, which is a `Int`. 
    As such, we write `n: Int`
-   The function returns an integer, which is why we end the definition with `: Int` before proceeding to the body of the function
-   The body of a function follows after the `=>`.
    The body of a function can either be an expression or a statement.

We must always include the types of the argument of a function.
We may however omit the return type of a function if it is inferrable from the body.
The return type can also be ommitted if the function does not return anything.
This is effectively the same as saying the function returns `None`.

### Default values

We can have default values:

    class MyClass
        def my_field := 5
        def my_method(x: Int, y: Int <- 2) => self.my_field := x + y

We can now call the method as such:
```
    def my_class := MyClass()
    my_class.my_method(10, 2)
```

### Default behaviour (Language feature ommitted for now, under review)

We can assign default behaviour to a method or function. 
To demonstrate this, we will use a toy factorial example. 
You might first write it as such:

    def factorial(n: Int): Int =>
        if   n = 0 then 1
        else n * self.factorial(n - 1) 

However, we could make this look much better with default behaviour. 

    def factorial (n: Int): Int => n * self.factorial (n - 1)  # for all other values of n, this function is called
    def factorial (0): Int      => 1                           # if n is 0, then this function is called instead

As long as a version exists of a function or method with arguments this is allowed.
