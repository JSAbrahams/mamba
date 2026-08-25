from abc import ABC


class MyType(ABC):
    pass


class MyType2(ABC):
    pass


class MyClass1(MyType, MyType2):
    other: int = None

    def __init__(self):
        MyType.__init__(self)
        MyType2.__init__(self)
