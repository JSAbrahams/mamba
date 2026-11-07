from typing import Tuple
def describe(p: Tuple[Tuple[int, str], int]) -> str: 
    match p:
        case ((a, s), b) if a + b > 3: 
            return s + "!"
        case _: 
            return "small"


print(describe(((1, "x"), 5)))
print(describe(((1, "x"), 1)))

