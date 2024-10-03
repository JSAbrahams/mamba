⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.2 📝 Data](README.md)

# 2.2.3 Operator Overloading

We can overload the following operators of the language:
-   `sqrt`
-   `+` and `-`
-   `*` and `/`
-   `^` and `mod`
-   `=` and `/=`
-   `<`, `<=`, `>`, and `>=`

We cannot overload the following operators, as these are used to compare instances directly and not their properties.
-   `is` and `isnt`
-   `isa` and `isnta`

Overloading operators gives us the ability to more concisely work with more complex objects.
To demonstrate the concept of operator overloading we will use the (incomplete) `Complex` class, which represent complex numbers.

Say we define a stateless `Complex` as such:
```
     stateless Complex(def real: Int, def imaginary: Int)     
         def + (other: Complex) => Complex(self real + other real, self imaginary + other imaginary)
         
         # we can also overload an unary opeator
         # when overloading, the default return value is the type itself, in this case Complex
         def sqrt () =>
            real      := sqrt (self real ^ 2 + self imaginary ^ 2)
            imaginary := sqrt (2 * self real * self imaginary)
            Complex(real, imaginary)
         
         def to_string() => "[self real] + [self imaginary]i"
```

Now we can use the `Complex` as follows:
```
    from complex use Complex

    def a := Complex(1, 2) # 1 + 2i
    def b := Complex(2, 3) # 2 + 3i

    # the `+` operator of Complex has been overloaded
    def c := a + b
    print c # prints 3 + 5i
```
