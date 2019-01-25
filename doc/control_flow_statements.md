# Control Flow Statements

## While

A while is useful when we want to execute a statement or expression, or block, multiple times as long as a certain
condition holds.

## For ... In

A for loop is useful when we want to iterate over items in a collection.


We can also do the following:

    for i in 0..10 print i
    
Which will print all numbers from `0` to `9`. Type `Int`, which is a `ClosedRange[Int]`, can be used in this manner 
using the `..` notation. It also possible to use a custom type `T` in this manner. In this case it should implement
`ClosedRange[T]`, and define the following method:

    to_range (from: T, to: T): List[T]