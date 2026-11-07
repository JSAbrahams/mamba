import sys
from typing import Tuple
sys.set_int_max_str_digits(0)
def safe(p: Tuple[int, int]) -> str: 
    match p:
        case (1, n) if 10 / n > 1: 
            return "unreachable"
        case _: 
            return "safe"


print(safe((0, 0)))

