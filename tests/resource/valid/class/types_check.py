from typing import Callable

print("my script")

class MyGeneric(str):
    def __init__(self):
        super(str, self).__init__()


class MyType:
    some_field: str = None

    def __init__(self, some_field: str):
        self.some_field = some_field


class SomeState(MyClass):
    def __init__(self):
        super(MyClass, self).__init__()


class OtherState(MyClass):
    def __init__(self):
        super(MyClass, self).__init__()


class SuperInterface:
    bar: int = None


class MyInterface(SuperInterface):
    required_field: int = None

    def __init__(self):
        super(SuperInterface, self).__init__()

    def higher_order(self) -> int:
        pass


class MyClass(MyType, MyInterface):
    my_field: int = None
    required_field: int = 100
    private_field: int = 20

    def __init__(self, my_field: int, other_field: str = "Hello"):
        super(MyType, self).__init__(other_field)
        super(MyInterface, self).__init__()
        self.my_field = my_field

    def fun_a(self): self.some_field = f"my field is {self.required_field}"

    def fun_b(self): print(f"this function is private: {self.private_field}!")

    def some_higher_order(self, fun: Callable[[int], int]) -> int: return fun(self.my_field)

    def higher_order(self) -> int: return self.some_higher_order(lambda x: x * 2)
