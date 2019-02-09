# Functions and Methods

The definition of a function or method very closely resembles that of a variable. So similar is it in fact that we use
the term definition for both interchangeably. We would define a function or method as follows:

    def f (x: Int): Int -> x + 1
    
The arguments must all have an explicit type. The return type may be omitted if it is inferable. Thus, we can also 
define the function `f` as such:

    def f (x: Int) -> x + 1

The below outline the specifics for each.

## Functions

Functions are found within a `script` or `util`.

## Methods

Methods are found within a `class`. Methods have access to the fields of an instance of a `class` by preceding these
with the `self` keyword. A method in a class would look as such:

    class MyClass
        def my_field <- 5
        
        def my_method(x: Int, y: Int) -> self my_field <- x + y
        def single_arg(x: Int) -> self my_field * x
        def no_args() -> self my_field <- 20 * my_field
        
And would be called as such:

    def my_class <- MyClass()
    my_class.my_method(30, 10)
    
### Postfix Notation

When a method or function has only one argument, we can use postfix notation when writing calling that function or 
method:

    my_function(10)
    my_function 10 # also works fine
    
    my_object.my_method(10)
    my_object.my_method 10  # this works too
    my_object my_method 10  # also works
    my_object my_method(10) # we can mix and match if we really want

### Default values

We can have default values:

    class MyClass
        def my_field <- 5
        
        def my_method(x: Int, y: Int <- 2) -> self my_field <- x + y

We can now call the method as such:

    def my_class <- MyClass()
    
    my_class.my_method(10, 2) # works fine
    my_class.my_method(10)    # exactly the same arguments as the function call above
    my_class my_method 10     # using postfix notation
    my_class my_method(10)    # also works

### Default behaviour

We can assign default behaviour to a method or function. To demonstrate this, we will use a toy factorial example. You
might first write it as such:

    def factorial(n: Int) ->
        if n eq 0 then 1
        else n * factorial (n - 1) 

However, we could make this look much better with default behaviour. 

    def factorial (n: Int) -> n * factorial (n - 1) # for all other values of n, this function is called
    def factorial (0)      -> 1                     # if n is 0, then this function is called instead

As long as a version exists of a function or method with arguments this is allowed.
