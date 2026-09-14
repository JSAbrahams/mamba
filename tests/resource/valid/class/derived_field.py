class Circle: 
    area: int = None
    def __init__(self, radius: int): 
        self.radius = radius
        self.area = self.radius * self.radius * 3


    def describe(self) -> int: 
        return self.area



c: Circle = Circle(2)
print(c.area)
print(c.describe())

