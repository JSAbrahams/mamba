from typing import Callable

class MyClass2(MyType):
    _z_modified = "asdf"
    _other_field = 10

    def __init__(self, other_field: int, z: int):
        super().__init__(self)
        if z > 10: raise Err("Something is wrong!")
        self.z_modified = z * SOME_CONSTANT

        something = {
            b : d, other : a
        }[c]

        a = None
        try:
            a = self.z_modified
        except Exception as err1:
            print("hey")
            print("there")
        except MyErr as err2:
            print("hoi")

    def connect(self):
        self.other_field = 200

    def fun_a(self):
        print(self)

    def _fun_b(self):
        print("this function is private!")

    def factorial(self, x: int =0):
        x * self.factorial(x - 1)

    def factorial_infinite(self, x: int):
        x * self.factorial(x)

    def a(self):
        self.a(self.b)

    def b(self, c: C):
        self.a(self.b(self.c))

    def c(self, d: D):
        self.a(self.b(self.c(d)))

    def some_higher_order(self, fun: Callable[[int], int]):
        0

    def fancy(self):
        self.some_higher_order(lambda x: x * 2) or 10