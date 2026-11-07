import sys
sys.set_int_max_str_digits(0)
def is_even(x: int) -> bool:
    return x % 2 == 0

def parity(x: int) -> str:
    match x:
        case n if is_even(n):
            return "even"
        case _:
            return "odd"

print(parity(4))
print(parity(5))

