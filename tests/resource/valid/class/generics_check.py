from collections import defaultdict
from typing import Callable
from typing import Optional


class Err1(Exception):
    def __init__(self, msg: str):
        super().__init__(msg)


class Err2(Exception):
    def __init__(self, msg: str):
        super().__init__(msg)


class MyType:
    super_field = None

    def __init__(self, super_field: str):
        self.super_field = super_field


class MyClass2(MyType):
    z_modified: str = "asdf"
    other_field: int = 10

    def __init__(self, other_field: int, z: int):
        super().__init__("the quick brown fox jumped over the slow donkey")
        if z > 10:
            raise Err1("Something is wrong!")
        self.z_modified = "fdsa"

        (a, b) = (10, 20)
        (a, b) = (30, 40)
        (a, b) = (0, 10)

        something = defaultdict(lambda: z + 100, {
            10: z + 10,
        })[z]

        my_bool = True
        other = {
            True: 2,
            False: 3,
        }[my_bool]

        a = None
        try:
            a = self.error_function()
        except Err1 as err1:
            print(err1)
            a = -1
        except Err2 as err2:
            print(err2)
            a = -2

        print(a)

    def error_function(self) -> int: return 200

    def connect(self): self.other_field = 200

    def fun_a(self): print(self)

    def _fun_b(self): print("this function is private!")

    def factorial(self, x: int = 0) -> int: return x * self.factorial(x - 1)

    def factorial_infinite(self, x: int) -> int: return x * self.factorial(x)

    def a(self) -> int: return self.b(10)

    def b(self, c: int) -> int: return self.a()

    def c(self, d: int) -> int: return self.b(self.c(20))

    def some_higher_order(self, fun: Callable[[int], int]) -> Optional[int]: return fun(10)

    def fancy(self) -> int: return self.some_higher_order(lambda x: x * 2) or 10
