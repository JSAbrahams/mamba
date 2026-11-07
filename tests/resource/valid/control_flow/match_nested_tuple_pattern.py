import sys
from typing import Tuple
sys.set_int_max_str_digits(0)
def describe(p: Tuple[Tuple[int, str], int]) -> str: 
    match p:
        case ((a, s), b) if a + b > 3: 
            return s + "!"
        case _: 
            return "small"


print(describe(((1, "x"), 5)))
print(describe(((1, "x"), 1)))

