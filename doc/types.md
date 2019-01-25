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
    
This solves our first issue. We now know what covered actually symbolises. But we still do not do any bounds checking.
Kilometer can be negative, or greater than the total. To this end, we can add ranges to the type definition itself:

    type Kilometer <- Int inrange 0..10 # excluding 10, so we have [0, 1, ..., 9]
    
Or

    type Kilometer <- Int inrange 0..=9 # including 9. Semantically speaking same as above but might be more clear
                                        # depending on the context.
    
Here, `inrange` uses the `to_range` method of the type `Int`. This can also be defined for user defined types. More can
be read about this in Control Flow Statement; For Loops. Of course, we may receive an error, so we must add an 
`OutOfBounds` error to the function signature:

    def distance_remaining(covered: Kilometer): Kilometer raises [OutOfBounds[Kilometer]] -> self total - covered
    
Due to the range of `Kilometer`, under the hood, it is desugared to the following:

    def distance_remaining(covered: Kilometer): Kilomter raises [OutOfBounds[Kilometer]] ->
        if (covered < 0) raise OutOfBounds(covered, 0)
        if (covered >= 10) raise OutOfBounds(covered, 0)
        
        return self.total - covered
