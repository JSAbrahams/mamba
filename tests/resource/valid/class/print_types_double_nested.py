class Y:
    def __init__(self, a: float):
        self.a = a


class X:
    y: Y = None

    def __init__(self, a: float):
        self.a = a
        self.y = Y(self.a)


x: X = X(10)
print(x.y.a)
