class A:
    c: C

class C:
    my_class: D = D()
    def my_field_accessor(self) -> D: self.my_class

class D:
    my_field: int = 10

a = A()
a.c.my_field_accessor().my_field = 20
