class Y:
    def __init__(self, a: float):
        self.a = a

class X:
    y = None
    def __init__(self, a: float):
        self.y = Y(a)

x: X = X(10)

x.y.a = x.y.a + 2
x.y.a = x.y.a - 3
x.y.a = x.y.a * 6

x.y.a = x.y.a / 7
x.y.a = x.y.a ** 2

x.y.a = x.y.a << 10
x.y.a = x.y.a >> 5
