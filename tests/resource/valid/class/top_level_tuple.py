import sys
sys.set_int_max_str_digits(0)
class A:
    a, b = (10, 100)


a: A = A()
print(a.a)
print(a.b)
