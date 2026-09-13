from abc import ABC
class MyType(ABC): 
    pass


class MyType2(ABC): 
    pass


class MyClass1(MyType, MyType2): 
    def __init__(self, other: int): 
        MyType.__init__(self)
        MyType2.__init__(self)
        self.other = other





