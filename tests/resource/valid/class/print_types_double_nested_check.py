class Y:
    def __init__(self, a: float):
        self.a = a

class X:
    y = None
    def __init__(self, a: float):
        self.y = Y(a)

x: X = X(10)
print(x.y.a)
