import sys
sys.set_int_max_str_digits(0)
class Point: 
    def __init__(self, x: int, y: int): 
        self.x = x
        self.y = y




p: Point = Point(3, 4)
print(p.x)

