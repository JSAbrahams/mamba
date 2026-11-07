import sys
sys.set_int_max_str_digits(0)
def sign(x: int) -> str:
    match x:
        case n if n < 0:
            return "negative"
        case n if n > 0:
            return "positive"
        case _:
            return "zero"


print(sign(-3))
print(sign(3))
print(sign(0))
