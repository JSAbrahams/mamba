import sys
from typing import Tuple
sys.set_int_max_str_digits(0)
def add_pair(m: str, p: Tuple[int, int]) -> int: 
    match p:
        case (m, n): 
            return m + n


print(add_pair("ignored", (2, 3)))

