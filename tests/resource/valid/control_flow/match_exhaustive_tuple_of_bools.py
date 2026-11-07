from typing import Tuple
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

