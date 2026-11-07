import sys
sys.set_int_max_str_digits(0)
def f(a="hello", b=True, c=5, d=5.0):
    print(a)


f()
f("hi", False, 1, 2.5)
