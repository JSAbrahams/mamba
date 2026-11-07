import sys
sys.set_int_max_str_digits(0)
def f() -> int: 
    return 3

match f():
    case 3: 
        a: str = "three"
    case _: 
        a: str = "other"

print(a)

