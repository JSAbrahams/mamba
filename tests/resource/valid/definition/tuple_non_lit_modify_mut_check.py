from typing import Tuple
class MyClass:
    my_field: int = 10

def f() -> Tuple[MyClass, int]:
    return (MyClass(), 10)

a, b = f()
a.my_field = 20
