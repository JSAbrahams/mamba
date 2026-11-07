from typing import Tuple
def first(p: Tuple[int, int]) -> int: 
    match p:
        case (0, _): 
            return 1
        case _: 
            return 2


def second(p: Tuple[int, str]) -> int: 
    match p:
        case (0, _): 
            return 1
        case _: 
            return 2


print(first((0, 5)))
print(second((3, "s")))

