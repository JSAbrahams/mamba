import sys
sys.set_int_max_str_digits(0)
class A:
    def __init__(self, b: B):
        self.b = b

class B:
    def my_c(self) -> C:
        return C()

class C:
    my_field: int = 10

a: A = A(B())
print(a.b.my_c().my_field)

