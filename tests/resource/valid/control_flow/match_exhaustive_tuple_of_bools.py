import sys
from typing import Tuple
sys.set_int_max_str_digits(0)
def both(p: Tuple[bool, bool]) -> str: 
    match p:
        case (True, True): 
            return "both"
        case (False, _): 
            return "not first"
        case (_, False): 
            return "not second"


print(both((True, True)))
print(both((False, True)))
print(both((True, False)))

