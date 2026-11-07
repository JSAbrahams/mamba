import sys
from typing import Optional
sys.set_int_max_str_digits(0)
def f(b: Optional[int]) -> int:
    return 1

print(f(None))
