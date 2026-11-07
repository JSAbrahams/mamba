import sys
sys.set_int_max_str_digits(0)
threshold: float = 5
def over(x: int) -> str:
    match x:
        case _ if x > threshold:
            return "over"
        case _:
            return "under"

print(over(10))
print(over(1))

