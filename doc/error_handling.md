# Error Handling

In some cases it may be that we might want to raise an error, if a user for instance passed a value which results in 
undefined behaviour (e.g. dividing by 0). Exception handling and `try` `catch` blocks are common in OOP languages. These
constructs however have been shown to be somewhat troublesome:

* When several lines of code are wrapped in a try catch block, we do not know which expression or statement is the one 
  which might throw an exception.
* Certain languages don't require all exceptions to be part of the function or method signature. This means that a 
  method call might result in an exception even if the source code does not reflect this. This means that the use of
  said method either has to either: 
    * Manually check that a method does indeed not throw an exception, which becomes exponentially more difficult when a
      method calls other methods, and so forth. 
    * Wrap all method calls in `try` `catch` blocks, which might often be unnecessarily and make the application 
      unnecessarily verbose.
    * Assume that the method will not throw an exception, which might be a source of bugs down the line and may result
      in runtime errors.

As such, we aim to address the above concerns by using a more explicit system of error handling outlined below. In 
general we:
    
* Handle errors where they occur
* We explicitly state that an expression might throw an error

File `my_err.mylang`:

    class MyErr(def msg: String) isa Err
        def to_string <- msg
    
Say we have the following function:

    def g(x: Int): Int raises[MyErr] -> if x is 10 then MyErr("x was 10") else x
    
    # We can also have a function that raises multiple types of errors
    def h(x: Int): Int raises[MyErr, OtherErr] -> if x > 10 then MyErr("bigger than 10") else OtherErr("or not")
    
Small side note: using the default behaviour feature of the language, we can rewrite `g` as such:

    def g 0                           -> MyErr("x was 10")
    def g(x: Int): Int raises [MyErr] -> x
    
Note that if the signature of a function states that a certain type of exception is thrown, it must be thrown at some
point, or we will get a type error:

    # type error! exception of type MyErr is can never be raised
    def no_err(x: Int): Int raises[MyErr] -> x + 1
 
When calling the function, it either has to be explicitly stated that it may raise an error, which is done like so:

    def l <- g(9) raises[MyErr]
    
Which must be added to the function signature if used within:

    # ommitting the raises in the signature of f would result in a type error!
    def f(x: Real): Real raises[MyErr] ->
        def a <- g(9) raises[MyErr] + 1.5
        a * 4.6
    
    # you can also put raises at the end of a statement or expression in a block.
    def h(x: Real): Real raises[MyErr] ->
        def a <- g(9) + 1.5 raises[MyErr] # If this statement would throw multiple types of exceptions, we would list 
                                          # them here
                                          
        # if the above raised an error, this would never get executed, as we are immediately 'raised' out of the 
        # function
        a * 2.3
    
Or, we explicitly handle it on site. We do this using the `handle when`, which matches the type of the returned value to
determine what to do. A good first step is to log the error. In this case, we simply print it:
    
    def l <- g(9) + 1.5 handle when
        err: MyErr -> println err
        ok         -> ok
        
    # here, l has type l?, as we don not know if an error occurred or not
    println "we don't know whether l is an Int or None"
    
That's slightly better, but we still don't know whether `l` is an `Int` or `None`. We modify the above code slightly
and get the following:

    def l <- g(9) handle when
        l: Int     -> println "we know for sure that l is an Int" # l 
        err: MyErr -> print err
 
In the above case, notice how we match on based on the type of the value the function `g` returned. Using this method, 
we can also handle different types of errors in different ways if we wish. If an error type is not covered, either by
matching using the `Err` type, or using a default branch, the compiler will raise a type error.
 
We may also return if we detect an error. In that case, the code after would only be executed if no error occurred:

    def l <- g(9) handle when
        err: MyErr ->
            print err
            return # or we may return the error: `return err`, or something else
            
    # if we execute this code we know for sure no error was thrown.
    # if an error was thrown this will not be executed at all
    println "[l] has type Int"
    
We can, instead of returning, also assign a default value to l. Though this should be done with care however. Assigning
to a definition if an error has occurred might bury the error, causing unexpected behaviour later during execution.

    def l <- g(9) handle when
        err: MyErr ->
            println err
            0
            
     # now, even if an error is thrown, l is assigned an integer, so we know that l is an Int
     println "[l] has type Int"
    
The above patterns ensure that error handling is always done explicitly and at the location where the error may occur.
There is no concept of runtime error. All errors must be explicit, and the type checker ensure that they are handled.
