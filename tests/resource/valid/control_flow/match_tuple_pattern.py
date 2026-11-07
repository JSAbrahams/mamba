from typing import Tuple
def quadrant(point: Tuple[int, int]) -> str: 
    match point:
        case (0, 0): 
            return "origin"
        case (x, y) if x > 0: 
            return f"right of the axis, at height {y}"
        case (x, y): 
            return f"left of the axis, at height {y}"


print(quadrant((0, 0)))
print(quadrant((2, 7)))
print(quadrant((-2, 7)))

