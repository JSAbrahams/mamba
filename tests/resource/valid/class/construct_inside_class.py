import sys
sys.set_int_max_str_digits(0)
class Point: 
    def __init__(self, x: int, y: int): 
        self.x = x
        self.y = y


    def origin() -> Point: 
        return Point(0, 0)



o: Point = Point.origin()
print(o.x)

