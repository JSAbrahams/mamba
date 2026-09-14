from typing import Tuple
def on_axis(point: Tuple[int, int]) -> str: 
    match point:
        case (0, _): 
            return "on the y axis"
        case (_, 0): 
            return "on the x axis"
        case _: 
            return "off both axes"


print(on_axis((0, 5)))
print(on_axis((5, 0)))
print(on_axis((5, 5)))

