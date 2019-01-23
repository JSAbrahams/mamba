# Operator Overloading

We can overload the following operators of the language:
* `sqrt`
* `+` and `-`
* `*` and `/`
* `^` and `mod`
* `eq` and `neq`
* `<`, `<=`, `>`, and `>=`

We cannot overload the following operators, as these are used to compare instances directly and not their properties.
* `is`
* `isnt`
* `isa`

Overloading operators gives us the ability to more concisely work with more complex objects.

To demonstrate the concept of operator overloading we will use the (incomplete) `complex` class, which represent complex 
numbers, to show how operator overloading may be used.

File `complex.mylang`:

     class Complex(def real: Int, def imaginary: Int)
     
     # the default return type for operators is the class itself
     def + (other: Complex) -> Complex(self real + other real, self imaginary + other imaginary)
     
     # we can also overload an unary opeator
     def sqrt () -> 
        real      <- sqrt (self real ^ 2 + self imaginary ^ 2)
        imaginary <- sqrt (2 * self real * self imaginary)
        Complex(real, imaginary)
     
     def to_string <- "[self real] + i[self imaginary]"
        
File `main.mylang`:

    from "complex" use Complex

    def a <- Complex(1, 2) # 1 + 2i
    def b <- Complex(2, 3) # 2 + 3i
    
    # the `+` operator of the class has been overloaded
    def c <- a + b
    println c # prints 3 + 5i
    