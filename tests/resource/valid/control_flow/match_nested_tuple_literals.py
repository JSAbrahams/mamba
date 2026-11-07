import sys
from typing import Tuple
sys.set_int_max_str_digits(0)
def describe(p: Tuple[Tuple[int, int], int]) -> str:
    match p:
        case ((0, 0), 0):
            return "all zero"
        case ((0, 0), _):
            return "flat"
        case _:
            return "other"

print(describe(((0, 0), 0)))
print(describe(((0, 0), 5)))
print(describe(((1, 2), 3)))

