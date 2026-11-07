import sys
sys.set_int_max_str_digits(0)
class Origin: 
    x: int = 0


def make() -> Origin: 
    return Origin()

print(make().x)

