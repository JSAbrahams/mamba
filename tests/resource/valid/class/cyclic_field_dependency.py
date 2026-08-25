class Cyclic:
    a: int = None
    b: int = None

    def __init__(self, x: int):
        self.x = x
        self.a = self.b + self.x
        self.b = self.a + self.x


c: Cyclic = Cyclic(3)
print(c.a)
