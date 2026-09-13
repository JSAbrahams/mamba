sqrt: int = 3
n: int = sqrt * sqrt
class Vec: 
    sqrt: int = None
    def __init__(self, x: int): 
        self.x = x
        self.sqrt = self.x




v: Vec = Vec(4)
print(n)
print(v.sqrt)

