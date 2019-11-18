class MyType:
    def fun_a(self): pass
    def factorial(self, x: int) -> int: pass


class MyClass1(MyType):
    other: int = None

    def __init__(self):
        super().__init__(f"asdf")
