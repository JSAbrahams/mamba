class X:
    def __init__(self, a):
        self.a = a

x = X(10)

x.a += 2
x.a -= 3
x.a *= 6

x.a /= 7
x.a **= 2

x.a <<= 10
x.a >>= 5
