import sys
sys.set_int_max_str_digits(0)
class X:
    a: int = 1
    def f(self):
        self.a = 2

def h(x: X):
    x.f()

y: X = X()
h(y)
print(y.a)

