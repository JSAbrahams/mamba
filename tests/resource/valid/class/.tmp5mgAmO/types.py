from typing import Callable
print("my script")
MyGeneric = str()
class MyType:


SomeState = MyClass()
OtherState = MyClass()
class SuperInterface:
    bar: int = None

class MyInterface(SuperInterface()):
    required_field: int = None
    def higher_order(self) -> int:
        pass

# this class has no state
class MyClass(MyType(other_field), MyInterface()):
    super(MyType, self).__init__(other_field)
    super(MyInterface, self).__init__()
    required_field: int = 100
    private_field: int = 20
    def fun_a(self):
        self.some_field = f"my field is {self.required_field}"

    def fun_b(self):
        print(f"this function is private: {self.private_field}!")

    def some_higher_order(self, fun: Callable[[int], int]) -> int:
        return fun(self.my_field)

    def higher_order(self) -> int:
        return self.some_higher_order(lambda x: x * 2)
