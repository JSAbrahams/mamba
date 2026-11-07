class A:
    b: B = None
    def __init__(self):
        self.b = B()

class B:
    c: C = None
    def __init__(self):
        self.c = C()

class C:
    d: D = None
    def __init__(self):
        self.d = D()

class D:
    e: int = 100

a: A = A()
a.b.c.d.e = 20

