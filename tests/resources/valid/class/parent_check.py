class MyType():
    my_field = None
    def fun_a(self): pass
    def factorial(self, x): pass

class MyClass1(MyType):
    def __init__(self):
        super().__init__(self, "asdf")

    other = None
