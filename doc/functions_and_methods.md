# Functions and Methods

The definition of a function or method very closely resembles that of a variable. So similar is it in fact that we use
the term definition for both interchangeably. We would define a function or method as follows:

    def f (x: Int): Int <- x + 1
    
The arguments must all have an explicit type. The return type may be omitted if it is inferrable. Thus, we can also 
define the function `f` as such:

    def f (x: Int) <- x + 1

The below outline the specifics for each.

## Functions

Functions are found within a `script` or `util`.

## Methods

Methods are found within a `class`. Methods have access to the fields of an instance of a `class` by preceding these
with the `self` keyword. A method in a class would look as such:

    class MyClass
        def my_field <- 5
        
        def my_method(x: Int, y: Int) <- self my_field <- x + y
        
And would be called as such:

    def my_class <- MyClass()
    my_class.my_method(30, 10)

In a method call, the dot `.` may be omitted. To the following would also be valid:

    my_class my_method(30, 10)
    
We may even omit the parenthesis that surround the method, provided that the method has no, or only takes one argument.
Say we expand our class:

    class MyClass
        def my_field <- 5
        
        def my_method(x: Int, y: Int) <- self my_field <- x + y
        def other_method(x: Int) <- self my_field <- x
        def no_args() <- self my_field <- 20 * my_field
        
Notice that we still need brackets even if we take no arguments. This is to differentiate methods from values.
Methods have side-effects, meaning that they can modify the state of the class, whereas values cannot inadvertently 
modify other values of the class. The brackets provide a clear visual distinction between the two.

Now, we can do the following:

    my_class.my_method(30, 2)  # with the . works fine
    my_class my_method(23, 10) # but without works to
    
    my_class other_method(210) # just like before
    my_class other_method 23   # no parenthesis here
    my_class no_args           # and here
