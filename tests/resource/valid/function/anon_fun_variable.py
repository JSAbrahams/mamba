import sys
from typing import Callable
sys.set_int_max_str_digits(0)
f: Callable[[int], int] = lambda a: a + 1
print(f(1))
