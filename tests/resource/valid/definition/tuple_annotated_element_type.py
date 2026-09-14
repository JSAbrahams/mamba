class MyClass: 
    my_field: int = 10


a, b = (10, "hi")
print(b)
c: int = a + 1
d, e = (MyClass(), 10)
print(d.my_field)

