class Point: 
    def __init__(self, x: int, y: int): 
        self.x = x
        self.y = y


    def new(x: int, y: int) -> Point: 
        return Point(x, y)

    def same_x(self, other: Point) -> bool: 
        return self.x == other.x



a: Point = Point.new(1, 2)
b: Point = Point.new(1, 9)
print(a.same_x(b))

