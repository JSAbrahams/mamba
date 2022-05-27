class Y:
    def __init__(self, a: float):
        self.a = a

class X:
    def __init__(self, a: float):
        self.y = Y(a)

x: X = X(10)

x.y.a = x.a + 2
x.y.a = x.a - 3
x.y.a = x.a * 6

x.y.a = x.a / 7
x.y.a = x.a ** 2

x.y.a = x.a << 10
x.y.a = x.a >> 5
