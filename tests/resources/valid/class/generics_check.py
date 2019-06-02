# This class has no state
class MyClass2(MyType):
    _z_modified = None
    _other_field = 10

    def __init__(self, other_field, z):
        super().__init__(self)
        if z > 10: raise Err("Something is wrong!")
        self.z_modified = z * SOME_CONSTANT # this is some code
        a = None

        try:
            a = self.z_modified
        except MyErr as err1:
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
        self.some_higher_order(lambda x: x * 2) or 10