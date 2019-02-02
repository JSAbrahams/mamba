# Undefined, or, Nullable Types

In some cases, it may be that a function returns nothing. Not to be confused with errors, which can be read about in 
"Errors".

However we wish to be explicit about a function which may return nothing, as the user of the function might expect a 
value.

For this we use the question mark symbol: `?`

Take the following:
    
    # type error! 'get' function might return nothing
    def my_function(set: Set[Int], str: String): Int -> set get str 

The type checker is complaining that the `get` function from `Set` might return `undefined`. To circumvent this, we make
the return type of the function nullable.

    def my_function(set: Set[String], str: String): String? -> set get str
    
Now when calling my function, I will either get an `Int` or a `undefined`. Because this is explicit, we know this at 
compile time. To cal a function on the resulting value, we may use the `?and` operator.

    def set ofmut <- { "hello" }
    def str_1 <- "hello"
    
    my_function(set, str_1) ? push "world" # only invoke push if my_function does not return undefined 
    my_function(set, str_1)? push "is"     # you can also place the ? directly after the returned value
    my_function(set, str_1)?.push "easy"   # if you use a dot, it must come after the ?, which is part of the value
    
If we try to call a function or access a definition of the function directly we get a type error:
    
    # type error! called `push` on an object which might be undefined
    my_function(set, str_1) push "world" 
   
### Default values
   
In some situations, we want to have a default value. In such situations, we use the `?or` operator. Note that both sides
of the operator must be of the same type.

    def world <- my_function(set, "world") ?or "world"
    
    # here, world is of type String
    
    def other <- my_function(set, "other")
    
    # here, other is of type String?, as we do not know whehter it is a String or undefined
    
You can also return `undefined` in a function:

    def special_function(x: Int): Int? -> if x > 10 then x else undefined
    