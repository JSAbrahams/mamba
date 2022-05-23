class X:
    def __init__(self, a: float):
        self.a = a

x = X(10)

x.a = x.a + 2
x.a = x.a - 3
x.a = x.a * 6

x.a = x.a / 7
x.a = x.a ** 2

x.a = x.a << 10
x.a = x.a >> 5
