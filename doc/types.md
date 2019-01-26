# Types

## Interfaces

## Type aliases

In some cases, for readability we might want to write a type alias. Say we have the following function:

    def distance_remaning(covered: Int): Int -> self total - covered
    
The above seems simple, but there are two issues:
* At a glance, we cannot know what covered symbolises. Kilometers, meters? We can of course rename the variable, but
  in certain situations this makes the code rather verbose.
* We do no bounds checking here. What if covered is more than the total, or negative? We could add these bounds checks
  to the method. However, this makes the method more verbose. Ideally, we want to method to express in a concise manner 
  what it does without having a majority of the method being error handling code.
  
To solve the above two issues, we can use type aliases. Observe the following:

    type Kilometer isa Int
    
Type `Kilometer` can do everything an `Int` can (we can use all the same operators), but using such an alias allows us
to more clearly express our ideas in the codebase without relying on documentation. (This is a recurring theme, source
code ideally should speak for itself without relying heavily on documentation.) We can rewrite the function as so:

    def distance_remaning(covered: Kilometer): Kilometer -> self total - covered

## Type Refinement

Type refinement expands upon type aliases by defining certain conditions an object must adhere to to be considered that
type. This can also be used to enforce pre-conditions of a function or method when used as a parameter, and 
post-conditions when used as the return type. This is akin to the philosophy of Design by Contract.

Say we have a function:

    def f (x: Int): Int -> 
        println("this number is even: [x]")
        x
    
In some situations, this function does not behave as we expect it to. It may print an uneven number. In such a
situation, we often turn to the design by contract philosophy, where a function has pre and post-conditions. There are
several traditional approaches to solving this problem, both if which are valid, though the preferred approach does 
depend on context:

Just return `x` if it is uneven and don't print anything using a simple `if`:

    def f (x: Int): Int -> 
        if x mod 2 isnt 0 return x
        println("this number is even: [x]")
        x
    
Raise and error if `x` is uneven:
    
    def f (x: Int): Int raises [Err] -> 
        if x mod 2 isnt 0 raise Err("Expected x to be even.")
        println("this number is even: [x]")
        x
    
However, in the above code, we see that writing pre-conditions can get out of hand, and we might want to use the same 
pre-conditions for multiple functions, which results in duplicate code. This is a situation where type aliases with 
conditions can come in handy. Type aliases encourages decentralisation. The logic of a type is closely linked to the
type itself, instead of having to manually check the a type adheres to certain conditions every time it is used.

We can use a trivial type `EvenNum` to demonstrate how one would use conditions in a type alias. Say we define the
type-alias `EvenNum`:

    type EvenNum isa Int where
        self mod 2 is 0 # we can list more conditions below this one. They must all evaluate to a boolean.
        
Which is the same as:
   
    type EvenNum isa Int where
        self mod 2 is 0 else Err()
        
We can also rewrite it for a more descriptive error message:

    type EvenNum isa Int where
        self mod 2 is 0 else Err("Expected an even number but was: [self]")

This defines all `Int`, or Integers, that are even. That is, the condition listed above holds. 

If we redefine the function as follows:

    # EvenNum has conditions, so we need to state that it may raise an error
    def g (x: EvenNum): Int raises[Err] -> 
        println("this number is even: [x]")
        x
    
Now, the actual type of the argument describes what conditions the argument adheres to, instead of having to manually
check these in the body of the function. We now know that these conditions hold in the body of the function. We can cast
any variable that is an `Int` to `EvenNum`. During casting, the defined conditions are checked, and the respective error
is thrown if a condition does not hold:

    # We can cast x to an EvenNum, which might give an error
    def x <- 10 # here x is an Int
    def y <- x as EvenNum raises [Err]
    def first <- g(y)
    
    # We can also cast inside the function argument if we want
    def z <- 10 # here x1 is an Int
    def second <- g(z as EvenNum) raises [Err]   
    
    # Or just say that the variable is an EvenNum upon instantiation
    def y: EvenNum <- 10 raises [Err]
    def third <- g(y)
    
    # We can also use the isa to check that the conditions holds without raising an error
    def a <- 9
    # notice how we don't have to cast a to an EvenNum if the condition holds.
    # We know that the then branch of the if is only executed if a is an EvenNum, so we assign it the type EvenNum
    fourth <- if a isa EvenNum then g(a) else 0
    
    # first, second, third and fourth all have type Int

We can also use it as a sort of post-condition of the function. We ensure that the function returns an `EvenNum`:

    def g (x: EvenNum): EvenNum raises[Err] -> 
        println("this number is even: [x]")
        def y <- x + some_other_function(x)
        y as EvenNum raises [Err]
        
Or:

    def g (x: EvenNum): EvenNum raises[Err] -> 
        println("this number is even: [x]")
        def y: EvenNum <- x + some_other_function(x) raises [Err]
        y

We can even ensure that the function never returns an error:

    def h (x: EvenNum): EvenNum -> 
        println("this number is even: [x]")
        def y <- x + some_other_function(x)
        if y isa EvenNum then
            y # type sensitive flow ensure that this is an EvenNum
        else 
            x # we know that x is an EvenNum

So now:

    def x <- 10 # here x is an Int
    def a <- g(x as EvenNum) raises [Err]
    
    # a has type EvenNum
    def b <- g(a) raises Err # we don't have to cast a to an EvenNum, it is already of that type
    
    def c <- h(x) # function h never raises an error
