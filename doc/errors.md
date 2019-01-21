# Errors

In some cases it may be that we might want to raise an error, if a user for instance passed a value which results in 
undefined behaviour (e.g. dividing by 0). Exception handling and `try` `catch` blocks are common in OOP languages. These
constructs however have been shown to be somewhat troublesome:

* When several lines of code are wrapped in a try catch block, we do not know which expression or statement is the one 
  which might throw an exception.
* Certain languages don't require all exceptions to be part of the function or method signature. This means that a 
  method call might result in an exception even if the source code does not reflect this. This means that the use of
  said method either has to either: 
    * Manually check that a method does indeed not throw an exception, which becomes exponentially more difficult when a
      method calls other method, and so forth. 
    * Wrap all method calls in `try` `catch` blocks, which might often be unnecessarily and make the application 
      unnecessarily verbose.
    * Assume that the method will not throw an exception, which might be a source of bugs down the line and may result
      in runtime errors.

As such, we aim to address the above concerns by using a more explicit system of error handling outlined below.

File `my_err.mylang`:

    class MyErr(def msg: String) isa Err
    
Say we have the following function:

    def g(x: Int): Int raises[MyErr] <- if x > 10 then x else MyErr("x was smaller than 10")
 
When calling the function, it either has to be explicitly stated that it may raise an error, which is done like so:

    def l <- g(9) raises[MyErr]
    
Which must be added to the function signature if used within:

    # ommitting the raises in the signature of f would result in a type error!
    def f(x: Real): Real raises[MyErr] <- g(9) raises[MyErr] + 1
    
Or, we explicitly handle it on site. We do this using the `handle when`, which matches the type of the returned value to
determine what to do. A good first step is to log the error. In this case, we simply print it:
    
    def l <- g(9) handle when
        err: MyErr -> println err
        
    # here, l has type l?, as we don not know if an error occurred or not
    println "we don't know whether l is an Int or None"
    
That's slightly better, but we still don't know whether `l` is an `Int` or `None`. We modify the above code slightly
and get the following:

    def l <- g(9) handle when
        l: Int   -> println "we know for sure that l is an Int" # l 
        err: MyErr -> print err
 
We may also do the following:

    def l <- g(9) hanle when
        err: MyErr ->
            print err
            return
            # or we return the error: `return err`
            # or we rerturn None: `return None`. Useful if an underlying process might generate an error but we only
            # wish to log it but don't care which.
            
    # if we execute this code we know for sure no error was thrown.
    # if an error was thrown this will not be executed at all
    println "[l] has type Int"
    
We can, instead of returning, also assign a default value to l:

    def l <- g(9) hanle when
        err: Err ->
            print err
            l <- 0
            
     # now, even if an error is thrown, l is assigned an integer, so we know it it an Int
     println "[l] has type Int"
    
The above patterns ensure that error handling is always done explicitly and at the location where the error may occur.
There is no concept of runtime error. All errors must be explicit, and the type checker ensure that they are handled.
