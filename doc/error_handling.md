# Error Handling

Errors are a fact of life. It may be the result of incorrect data, user input, or a programming mistake. Error handling,
ideally, should be done in an explicit manner. However, at the same time, error handling code should not become overly
verbose as it might obfuscate the actual relevant parts of the codebase which perform the actual calculations. Thus, a
balance must be reached.

In some cases it may be that we might want to raise an error. Exception handling and `try` `catch` blocks are common in 
modern languages. These constructs however have been shown to be somewhat troublesome:

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
    
* Wish to handle errors where the occur in an explicit manner, or,
* We explicitly state that an expression or statement (or function or method) might throw an error. This creates a 
  visual stack trace within the codebase itself, so anyone who reads the code knows where an error might originate from
  without even having to compile the and run the code.

### Raises, and Result

Say we have the following error class:

    class MyErr(def msg: String) isa Err
        def to_string <- msg
    
And the following functions elsewhere (not within the error class):

    def g (x: Int): Int raises[MyErr] -> if x is 10 then MyErr("x was 10") else x
    
    # We can also have a function that raises multiple types of errors
    def h (x: Int): Int raises[MyErr, OtherErr] -> if x > 10 then MyErr("bigger than 10") else OtherErr("or not")
    
We can also use the `Result` type to define a possible return type and error pair:

    def g (x: Int): Result[Int, MyErr] -> if x is 10 then MyErr("x was 10") else x
    
    # We can also have a function that raises multiple types of errors
    def h (x: Int): Result[Int, [MyErr, OtherErr]] -> if x > 10 then MyErr("bigger than 10") else OtherErr("or not")
    
The first way of writing is preferred, as this more clearly separates the return type and possible errors that may be 
raised. However, using Result may be better in some other situations. For instance, it allows us to use the type alias 
feature of the language, which can be convenient in certain situations, such as when we wish to enforce consistency. See
"Types" for a more in-depth explanation. A trivial case would be:

    type MyResult <- Result[Int, MyErr]
    
    def g (x: Int): MyResult -> if x is 10 then MyErr("x was 10") else x
    
Small side note: using the default behaviour feature of the language, we can rewrite `g` as such:

    def g (x: Int): Int raises [MyErr] -> x
    def g 0                            -> MyErr("x was 10")
    
Note that if the signature of a function states that a certain type of exception is thrown, it must be thrown at some
point, or we will get a type error:

    # type error! exception of type MyErr is can never be raised
    def no_err(x: Int): Int raises[MyErr] -> x + 1
 
When calling the function, it either has to be explicitly stated that it may raise an error, which is done like so:

    def l <- g(9) raises[MyErr]
    
Which must be added to the function signature if used within:

    # ommitting the raises in the signature of f would result in a type error!
    def f (x: Real): Real raises[MyErr] ->
        def a <- g(9) raises[MyErr] + 1.5
        a * 4.6
    
    # you can also put raises at the end of a statement or expression in a block.
    def h (x: Real): Real raises[MyErr] ->
        def a <- g(9) + 1.5 raises[MyErr] # If this statement would throw multiple types of exceptions, we would list 
                                          # them here
                                          
        # if the above raised an error, this would never get executed, as we are immediately 'raised' out of the 
        # function
        a * 2.3
    
### Handle    
    
We can also explicitly handle it on site. We do this using the `handle when`, which matches the type of the returned
value to determine what to do. A good first step is to log the error. In this case, we simply print it using `println`:
    
    def l <- g(9) + 1.5 handle when
        err: MyErr -> println err
        
    # here, l has type l?, as we don not know if an error occurred or not
    println "we don't know whether l is an Int or None"
    
The above would desugar to the following:

    def l <- g(9) + 1.5 handle when
        err: MyErr -> 
            println err
            None # we return none, since the error case originally ended with a statement, here a print line
        ok         -> ok

    # here, l has type l?, as we don not know if an error occurred or not
    println "we don't know whether l is an Int or None"
    
That's slightly better, but we still don't know whether `l` is an `Int` or `None`. We modify the above code slightly
and get the following:

    def l <- g(9) handle when
        l: Int     -> println "we know for sure that l is an Int here"
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
    println "[l] has type Int and not Int?"
    
We can, instead of returning, also assign a default value to l. This should be done with care however. Assigning
to a definition if an error has occurred might bury the error, causing unexpected behaviour later during execution.

    def l <- g(9) handle when
        err: MyErr ->
            println err
            0
            
     # now, even if an error is thrown, l is assigned an integer, so we know that l is an Int
     println "[l] has type Int"
    
### Retry

Say we have a non-deterministic function `connect_to_server`, which tries to connect with an online server. It may be
that if we don't succeed the first time we may want to try another time. Instead of wrapping everything in a while loop,
we can use the `retry` keyword:

    def tries <- 0
    def l <- g(9) handle when
            err: MyErr ->
                println err
                tries += 1
                retry if tries <= 10
    
The above patterns ensure that error handling is always done explicitly and at the location where the error may occur.
There is no concept of runtime error. All errors must be explicit, and the type checker ensure that they are handled.
