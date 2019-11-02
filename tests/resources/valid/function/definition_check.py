from typing import Optional
from typing import Tuple
from typing import Callable

def fun_a() -> Optional[int]:
    print(11)
    if True and True:
        print(f"hello")
    if False or True:
        print(f"world")
    a = None or 1 if True else 11
    return 10 if True else None

def fun_b(b: int): print(b)

def fun_c(d: Tuple[str, int]): print(d)

def fun_d(h: Callable[[str, str], int])-> Optional[int]: return h(f"hello", f"world")

def fun_e(m: int, o: Tuple[str, str], r: Callable[[int, Tuple[str, str]], int]) -> int: return r(m, o)

def fun_v(y: str, ab: Callable[[str], Callable[[str], bool]]) -> Callable[[str], bool]: return ab(y)

class MyClass:
    a: int = None
    b: int = None
    def __init__(self, a: int, b: int):
        self.a: int = a
        self.b: int = b

    def some_function(self, c: int) -> int:
        d = 20
        d = 10 + 30
        return c + 20

    def __add__(self, other: MyClass) -> MyClass: return MyClass(self.a + self.b + other.some_function(self.a), self.b)
    def __sub__(self, other: MyClass) -> MyClass: return self + other
    def __mul__(self, other: MyClass) -> MyClass: return MyClass(self.a * other.b, self.b)
    def __truediv__(self, other: MyClass) -> MyClass: return MyClass(self.a // other.b, self.b)
    def __floordiv__(self, other: MyClass) -> MyClass: return MyClass(self.a // other.b, self.b)
    def __pow__(self, other: MyClass) -> MyClass: return MyClass(self.a ** other.b, self.b)

    def __eq__(self, other: MyClass) -> bool: return self.a == other.b
    def __gt__(self, other: MyClass) -> bool: return self.a > other.b
    def __lt__(self, other: MyClass) -> bool: return self.a < other.b

    def sqrt(self) -> MyClass: return MyClass(self.a // self.b, self.a // self.b)
    def __mod__(self, other: MyClass) -> MyClass: return MyClass(self.a % self.b, self.b)

def factorial(x: int) -> int: return x * factorial(x - 1)

def some_higher_order(f: Callable[[int], int], x: int)-> int: return f(x)
