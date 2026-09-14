class Circle: 
    diameter: float = None
    def __init__(self, radius: float): 
        self.radius = radius
        self.diameter = self.radius * 2.0


    def area(self) -> float: 
        return self.radius * self.radius * 3.14159



c: Circle = Circle(2.0)
print(c.diameter)
print(c.area())

