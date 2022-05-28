class MyType:
    def __init__(self, a: str):
        self.a = a

class MyType2:
    def __init__(self, b: str):
        self.b = b

class MyClass1(MyType, MyType2):
    other: int = None

    def __init__(self):
        MyType.__init__(self, "asdf")
        MyType2.__init__(self, "qwerty")
