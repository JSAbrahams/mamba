from typing import Generic


class MyClass(Generic[T]):
    def f(x: T) -> T:
        return x


x = MyClass()
