from abc import ABC, abstractmethod


class MyType(ABC):
    @abstractmethod
    def fun_a(self): pass

    @abstractmethod
    def factorial(self, x: int) -> int: pass


class MyClass1(MyType):
    other: int = None

    def __init__(self):
        MyType.__init__(self)

    def fun_a(self):
        print("hello")

    def factorial(self, x: int) -> int:
        return x * 1
