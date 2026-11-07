import sys
from typing import Tuple
sys.set_int_max_str_digits(0)
b: set[Tuple[int, int]] = {(1, 4), (2, 5)}
for (first, second) in b:
    print(first + second)
