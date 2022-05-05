from typing import Callable


MyGeneric = str

class MyType:
    def __init__(self, some_field: str):
        self.some_field = some_field


SomeState = MyClass
OtherState = MyClass


class SuperInterface:
    bar: int = None


class MyInterface(SuperInterface):
    required_field: int = None

    def higher_order(self) -> int:
        pass


class MyClass(MyType, MyInterface):
    required_field: int = 100
    private_field: int = 20

    def fun_a(self): self.some_field = f"my field is {self.required_field}"

    def fun_b(self): print(f"this function is private: {self.private_field}!")

    def some_higher_order(self, fun: Callable[[int], int]) -> int: return fun(self.my_field)

    def higher_order(self) -> int: return self.some_higher_order(lambda x: x * 2)

    def __init__(self, my_field: int, other_field: str = "Hello"):
        super(MyType, self).__init__(other_field)
        super(MyInterface, self).__init__()
        self.my_field = my_field
