from typing import Callable


class MyGeneric(str):
    def __init__(self):
        super().__init__()


class MyType:
    some_field: str = None

    def __init__(self, some_field): self.some_field = some_field


SomeState = MyClass
OtherState = MyClass


class SuperInterface:
    bar: int = None


class MyInterface(SuperInterface):
    required_field: int = None

    def higher_order(self) -> int: pass


class MyClass2(MyType, MyInterface):
    required_field: int = 100
    _private_field: int = 20
    my_field: str = None

    def __init__(self, my_field, other_field: str = f"Hello"):
        super().__init__(other_field)
        self.my_field = my_field

    def fun_a(self): self.some_field = f"my field is {self.required_field}"

    def _fun_b(self): print(f"this function is private: {self._private_field}!")

    def some_higher_order(self, fun: Callable[[int], int]) -> int: return fun(self.my_field)

    def higher_order(self) -> int: return self.some_higher_order(lambda x: x * 2)
