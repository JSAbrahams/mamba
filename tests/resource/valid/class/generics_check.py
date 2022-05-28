from typing import Callable
from typing import Optional


class Err1(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)


class Err2(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)

class MyType:
    def __init__(self, super_field: str):
        self.super_field = super_field


class MyClass2(MyType):
    z_modified: str = "asdf"
    other_field: int = 10

    def error_function(self) -> int: 200

    def connect(self): self.other_field = 200

    def _fun_b(self): print("this function is private!")

    def factorial(self, x: int = 0) -> int: x * self.factorial(x - 1)

    def factorial_infinite(self, x: int) -> int: x * self.factorial(x)

    def a(self) -> int: self.b(10)

    def b(self, c: int) -> int: self.a()

    def c(self, d: int) -> int: self.b(self.c(20))

    def some_higher_order(self, fun: Callable[[int], int]) -> Optional[int]: fun(10)

    def fancy(self) -> Optional[int]: self.some_higher_order(lambda x: x * 2)

    def __init__(self, other_field: int, z: int):
        MyType.__init__(self, "the quick brown fox jumped over the slow donkey")

        if z > 10:
            raise Err1("Something is wrong!")
        self.z_modified = "fdsa"

        a, b = (10, 20)
        a, b = (30, 40)
        (a, b) = (0, 10)

        my_bool: bool = True

        a = None
        try:
            a: int = self.error_function()
        except Err1 as err1:
            print(err1)
            a = -1
        except Err2 as err2:
            print(err2)
            a = -2
