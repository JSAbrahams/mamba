class Matrix2x2: 
    def __init__(self, a: float, b: float, c: float, d: float): 
        self.a = a
        self.b = b
        self.c = c
        self.d = d


    def identity() -> Matrix2x2: 
        return Matrix2x2(1.0, 0.0, 0.0, 1.0)



m: Matrix2x2 = Matrix2x2.identity()
print(m.a)

