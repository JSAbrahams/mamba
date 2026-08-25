class Point:
    sum: int = None
    doubled: int = None

    def __init__(self, x: int):
        self.x = x
        self.doubled = self.x * 2
        self.sum = self.x + self.doubled


p: Point = Point(3)
print(p.sum)
