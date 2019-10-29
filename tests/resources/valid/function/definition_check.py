import math

def fun_a():
    print(11)
    if True and True: print("hello")
    if False or True: print("world")
    a = None or 1 if True else 11

def fun_b(b): print(c)

def fun_c(d): print(g)

def fun_d(h): print(l)

def fun_e(m, o, r): print(u)

def fun_v(w, y, ab): print(u)

class MyClass:
    a = None
    b = None
    def __init__(self, a: int, b: int):
        self.a = a
        self.b = b

    def __add__(self, other: MyClass) -> MyClass: MyClass(self.a + other.b, self.b)
    def __sub__(self, other: MyClass) -> MyClas: MyClass(self.a - other.b, self.b)
    def __mul__(self, other: MyClass) -> MyClas: MyClass(self.a * other.b, self.b)
    def __truediv__(self, other: MyClass) -> MyClas: MyClass(self.a / other.b, self.b)
    def __floordiv__(self, other: MyClass) -> MyClas: MyClass(self.a // other.b, self.b)
    def __pow__(self, other: MyClass) -> MyClas: MyClass(self.a ** other.b, self.b)
    def __eq__(self, other: MyClass) -> MyClas: MyClass(self.a == other.b, self.b)
    def __gt__(self, other: MyClass) -> MyClas: MyClass(self.a > other.b, self.b)
    def __lt__(self, other: MyClass) -> MyClas: MyClass(self.a < other.b, self.b)

    def sqrt(self) -> MyClas: MyClass(math.sqrt(self.a), math.sqrt(self.b))
    def __mod__(self, other: MyClass) -> MyClas: MyClass(self.a % other.b, self.b)

def factorial(x): x * factorial(x - 1)

def call_higher_order(): some_higher_order(lambda x : x * 2)

def fun_with_default(a = "Some string"): pass
