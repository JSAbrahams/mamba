class Point: 
    def __init__(self, x: int, y: int): 
        self.x = x
        self.y = y


    def new(x: int, y: int) -> Point: 
        return Point(x, y)

    def origin() -> Point: 
        return Point(0, 0)



def shifted() -> Point: 
    return Point.new(1, 2)

p: Point = shifted()
print(p.x)
o: Point = Point.origin()
print(o.y)

