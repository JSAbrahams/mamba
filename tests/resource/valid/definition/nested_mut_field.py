class B:
    my_num: int = 10

class A:
    b: B = None


a: A = A()
a.b.my_num = 20
