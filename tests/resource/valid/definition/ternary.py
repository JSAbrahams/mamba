import sys
sys.set_int_max_str_digits(0)
if True:
    print(10)
else:
    print(20)

a: int = 20 if False else 30
