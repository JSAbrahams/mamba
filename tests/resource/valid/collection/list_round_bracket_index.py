import sys
sys.set_int_max_str_digits(0)
a: list[int] = [1, 2, 3]
a[0] = 9
a[1] = a[0] + 1
print(a[2])
