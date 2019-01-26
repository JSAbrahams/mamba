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

    type Kilometer <- Int
    
Type `Kilometer` can do everything an `Int` can (we can use all the same operators), but using such an alias allows us
to more clearly express our ideas in the codebase without relying on documentation. (This is a recurring theme, source
code ideally should speak for itself without relying heavily on documentation.) We can rewrite the function as so:

    def distance_remaning(covered: Kilometer): Kilometer -> self total - covered

## Type Refinement

Type refinement expands upon type aliases by defining certain conditions an object must adhere to to be considered that
type. This can also be used to enforce pre-conditions of a function or method when used as a parameter, and 
post-conditions when used as the return type. This is akin to the philosophy of Design by Contract.

### Conditions

Say we have a function:

    def f (x: Int): Int -> 
        println("this number is even: [x]")
        x
    
In some situations, this function does not behave as we expect it to. It may print an uneven number. In such a
situation, we often turn to the design by contract philosophy, where a function has pre and post-conditions. There are
several traditional approaches to solving this problem, both if which are valid, though the preferred approach does 
depend on context:
* Just return `x` if it is uneven and don't print anything using a simple `if`:


    def f (x: Int): Int -> 
        if x mod 2 isnt 0 return x
        println("this number is even: [x]")
        x
    
* Raise and error if `x` is uneven

    
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

    type EvenNum <- Int where
        self mod 2 is 0 # we can list more conditions below this one. They must all evaluate to a boolean.
        
Which is the same as:
   
    type EvenNum <- Int where
        self mod 2 is 0 else Err()
        
We can also rewrite it for a more descriptive error message:

    type EvenNum <- Int where
        self mod 2 is 0 else Err("Expected an even number but was: [x]")

This defines all `Int`, or Integers, that are even. That is, the condition listed above holds. 

If we redefine the function as follows:

    # EvenNum has conditions, so we need to state that it may raise an error
    def g (x: EvenNum): Int raises[Err] -> 
        println("this number is even: [x]")
        x
    
Now, the actual type of the argument describes what conditions the argument adheres to, instead of having to manually
check these in the body of the function. Effectively, the body of the function is only executed of the pre-condition
holds. So, the above desugars to:

    def g (x: Int): Int raises[Err] -> 
        if not x mod 2 is 0 return Err("Expected an even number but was: [x]")
        
        println("this number is even: [x]")
        x

We can also use it as a post-condition of the function:

    def g (x: EvenNum): EvenNum raises[Err] -> 
        println("this number is even: [x]")
        x

Which would desugar to:

    def g (x: Int): Int raises[Err] -> 
        if not x mod 2 is 0 return Err("Expected an even number but was: [x]")
        
        println("this number is even: [x]")
        
        if not x mod 2 is 0 return Err("Expected an even number but was: [x]")
        x

### Range

This solves our first issue. We now know what covered actually symbolises. But we still do not do any bounds checking.
Kilometer can be negative, or greater than the total. To this end, we can add ranges to the type definition itself:

    type Kilometer <- Int where
        inrange 0 to 10 # excluding 10, so we have [0, 1, ..., 9]
    
Or:

    type Kilometer <- Int where
        inrange 0 toincl 9 # including 9. Semantically speaking same as above but might be more clear
                           # depending on the context.
                           
Which is the same as writing:

    type Kilometer <- Int where
        inrange 0 to 10 else RangeErr("[self] is out of bounds. (From: [0.to_string], To: [10.to_string]))")
      
In the above example, `RangeErr` is a type of `Err`. We can also state that we wish to return our own type:

    type Kilomtere <- Int where
        inrange 0 to 10 else MyErr(self)
    
Here, `inrange` uses the `to_range` method of the type `Int`. This can also be defined for user defined types. More can
be read about this in Control Flow Statement; For Loops. Of course, we may receive an error, so we must add an 
`OutOfBounds` error to the function signature:

    def distance_remaining(covered: Kilometer): Kilometer raises [OutOfBounds[Kilometer]] -> self total - covered
    
Now, because type `Kilometer`, under the hood, the above is desugared to the following:

    def distance_remaining(covered: Kilometer): Kilomter raises [OutOfBounds[Kilometer]] ->
        if (covered < 0) raise OutOfBounds(covered, 0)
        if (covered >= 10) raise OutOfBounds(covered, 0)
        
        return self.total - covered
        
This can also be combined with the Conditions defined above as such:

    type EvenKilometer <- Int where
        inrange 0 to 10
        self mod 2 is 10 else Err("Expected an even number kilometers but was: [x]")

Or, if `Kilometer` has already been defined, we can extend from it, so we effectively inherit all its conditions. So we
can then rewrite the above as such:

    type EvenKilometer <- Kilomter where
        self mod 2 is 10 else Err("Expected an even number kilometers but was: [x]")
