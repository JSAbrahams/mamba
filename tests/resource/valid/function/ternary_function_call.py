import sys
sys.set_int_max_str_digits(0)
def my_fun(x: int):
    print(x)

my_fun(1 if True else 2)
