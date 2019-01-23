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

In a method call, the dot `.` may be omitted, if there is only a single argument. Better still, when a function or 
method has a single argument (or no arguments), we may even omit the brackets when calling the function. So we get the 
following:

    my_class.my_method(30, 10) # need dot here
    my_class single_arg(30)      # don't need it here though
    my_class single_arg 30       # I can even omit the brackets if there is only one argument
    my_class no_args             # or no argument for that matter
        
Notice that we still need brackets even if we take no arguments. This is to differentiate methods from values.
Methods have side-effects, meaning that they can modify the state of the class, whereas values cannot inadvertently 
modify other values of the class. The brackets provide a clear visual distinction between the two.

Now, we can do the following:

    my_class.my_method(30, 2)  # with the . works fine
    my_class my_method(23, 10) # but without works to
    
    my_class other_method(210) # just like before
    my_class other_method 23   # no parenthesis here
    my_class no_args           # and here

### Default values

We can have default values:

    class MyClass
        def my_field <- 5
        
        def my_method(x: Int, y: Int = 2) -> self my_field <- x + y

We can now call the method as such:

    def my_class <- MyClass()
    
    my_class.my_method(10, 2) # works fine
    my_class my_method(10)    # exactly the same arguments as the function call above!
    my_class my_method 10     # now we don't even need the parenthesis if we want

### Default behaviour

We can assign default behaviour to a method or function. To demonstrate this, we will use a toy factorial example. You
might first write it as such:

    factorial(n: Int) ->
        if n eq 0
        then 1
        else n * factorial (n - 1) 

However, we could make this look much better with default behaviour. 

    factorial 0        -> 1                     # if n is 0, then this function is called instead of the one below
    factorial (n: Int) -> n * factorial (n - 1) # for all other values of n, this function is called

As long as a version exists of a function or method with arguments this is allowed.

### Extensions

Sometimes, you might want to add a definition to a class. For this we have extensions, which allow us to add publicly
visible definition to a class locally. For safety reasons however, we cannot however overwrite definitions.

Say that we have class `Graph`. The writer of the class wrote a method called `node_count` which returns the amount of
nodes in a `Graph`. We then decide to use this functionality using the `|` size operators.

    # we defined a graph before and did some operations
    
    # error! graph does not define 'size'
    def size_of_graph <- |graph|
    println "The size of the graph is now [size_of_graph] nodes"
    
We get an error however: `error! graph does not define 'size'`. Now, we could decide to use the `node_count` definition,
but this might make our code look a bit messy or inconsistent if we have used the `|` operators elsewhere. Instead, we
do the following: At the top of our file, we write, after the imports:

    Graph.size <- self node_count

Now, have defined the `size` property for Graphs. Note that `self` is bound in this case, self refers to a particular
instance of the graph. We can now write the same code again, without getting an error:

    def size_of_graph <- |graph|
    println "The size of the graph is now [size_of_graph] nodes"
