import sys
from typing import Any
sys.set_int_max_str_digits(0)
a: list[int] = [0, 2, 51]
b: list[str] = ["list", "of", "strings"]
empty_list: list[Any] = []
a_positive: list[int] = [x for x in a if x > 0]
print(a[0])
print(b[1])

