from abc import ABC


class MyType(ABC):
    pass


class MyClass2:
    z_modified: str = "asdf"
    other_field: int = None

    def __init__(self, other_field: int, z: int):
        self.other_field = other_field
        self.z = z
        self.other_field = self.z + self.other_field
