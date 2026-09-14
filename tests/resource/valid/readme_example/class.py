class MatrixErr(Exception): 
    def __init__(self, message: str): 
        Exception.__init__(self, message)




class Matrix2x2: 
    def __init__(self, a: float, b: float, c: float, d: float): 
        self.a = a
        self.b = b
        self.c = c
        self.d = d


    def contents(self) -> list[float]: 
        return [self.a, self.b, self.c, self.d]

    def trace(self) -> float: 
        return self.a + self.d

    def determinant(self) -> float: 
        return self.a * self.d - self.b * self.c

    def solve(self, u: float, v: float) -> list[float]: 
        det: float = self.determinant()
        if det == 0.0:
            raise MatrixErr("Determinant is zero.")
        x: float = u * self.d - self.b * v
        y: float = self.a * v - u * self.c
        return [x / det, y / det]


    def scale(self, factor: float): 
        self.a = self.a * factor
        self.b = self.b * factor
        self.c = self.c * factor
        self.d = self.d * factor


    def reset(self): 
        self.a = 1.0
        self.b = 0.0
        self.c = 0.0
        self.d = 1.0





