import sys
from typing import Optional, Tuple
sys.set_int_max_str_digits(0)
b: Optional[bool] = None
p: Tuple[Optional[bool], int] = (b, 1)
match p:
    case (True, _):
        print(1)
    case (False, _):
        print(2)
    case (None, _):
        print(3)
