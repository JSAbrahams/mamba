from typing import Generic


class MyClass(Generic[T, T2]):
    def f(x: T) -> T2:
        return f"{x} is something"


x = MyClass[Int, String]()
