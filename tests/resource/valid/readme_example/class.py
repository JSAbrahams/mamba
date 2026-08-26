class MatrixErr(Exception): 
    def __init__(self, message: str): 
        Exception.__init__(self, message)




class Matrix2x2: 
    def __init__(self, a: int, b: int, c: int, d: int): 
        self.a = a
        self.b = b
        self.c = c
        self.d = d


    def contents(self) -> list[int]: 
        return [self.a, self.b, self.c, self.d]

    def trace(self) -> int: 
        return self.a + self.d

    def determinant(self) -> int: 
        return self.a * self.d - self.b * self.c

    def scale(self, factor: int): 
        self.a = self.a * factor
        self.b = self.b * factor
        self.c = self.c * factor
        self.d = self.d * factor


    def reset(self): 
        self.a = 1
        self.b = 0
        self.c = 0
        self.d = 1





