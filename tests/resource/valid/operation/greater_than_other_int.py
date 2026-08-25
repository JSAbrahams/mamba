class MyClass:
    def __init__(self, a: int):
        self.a = a

    def f(self, other: MyClass) -> bool:
        return self.a > other.a

a: MyClass = MyClass(10)
b: MyClass = MyClass(20)

a.f(b)
