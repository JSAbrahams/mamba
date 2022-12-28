class MyClass:
    def __init__(self, a: int):
        self.a = a

    def f(self) -> bool:
        return self.a > 10

a: MyClass = MyClass(10)
a.f()
