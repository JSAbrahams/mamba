from Math import abs
import something

class MyClass2(MyType):

    def __init__(self, my_field):
        super().__init__(self, my_field)

    _z_modified = None
    _other_field = 10

    def connect(self):
        self.other_field = 200

    def fun_a(self):
        print(self)

    def _fun_b(self):
        print("this function is private!")

    def factorial(self, x=0):
        x * self.factorial(x - 1)

    def factorial_infinite(self, x):
        x * self.factorial(x)

    def a(self):
        self.a(self.b)

    def b(self, c):
        self.a(self.b(self.c))

    def c(self, d):
        self.a(self.b(self.c(d)))

    def some_higher_order(self, fun):
        0

    def fancy(self):
        self.some_higher_order(lambda x: x * 2)
