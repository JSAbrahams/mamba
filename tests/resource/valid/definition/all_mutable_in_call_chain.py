class A :
    b: B = B()

class B :
    c: C = C()

class C :
    d: D = D()

class D :
    e: int = 100

a = A()
a.b.c.d.e = 20
