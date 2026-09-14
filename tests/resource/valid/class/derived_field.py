class Circle: 
    diameter: int = None
    def __init__(self, radius: int): 
        self.radius = radius
        self.diameter = self.radius * 2


    def describe(self) -> int: 
        return self.diameter



c: Circle = Circle(2)
print(c.diameter)
print(c.describe())

