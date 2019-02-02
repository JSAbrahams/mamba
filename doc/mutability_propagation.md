# Mutability Propagation

Mutability is one of the key features of the language. It ensures that if a variable is declared immutable, it is
indeed immutable, including its fields if it contains any. 

When type checking a method of a class, it the type checker detects that the function modifies a field of that class,
it will mark that method. That method may then only be used when the instance has been declared as mutable.

File `my_class.mylang`:

    class MyClass
        def private field <- 10
        
        # modifies a field of the class        
        def f () -> field <- 20
        
        # does not modify a field of the class
        def g () -> field * 10

File `main.mylang`:
    
    def mut a <- MyClass()
    def b <- MyClass()
    
    print a.field # prints 10
    a.f()
    print a.field # prints 20
    
    b.f() # type error! f modifies a field of the instance but instance is immutable
    