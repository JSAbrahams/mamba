class MyType:
    def __init__(self, super_field: str):
        self.super_field = super_field

class MyClass2(MyType):
    z_modified: str = "asdf"
    other_field: int = 10

    def __init__(self, other_field: int, z: int):
        MyType.__init__(self, "the quick brown fox jumped over the slow donkey")
        self.other_field = z + other_field
