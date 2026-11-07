from typing import Tuple
def safe(p: Tuple[int, int]) -> str: 
    match p:
        case (1, n) if 10 / n > 1: 
            return "unreachable"
        case _: 
            return "safe"


print(safe((0, 0)))

