class C: 
    def double(self, x: int) -> int: 
        return x * 2

    def quadruple(self, x: int) -> int: 
        return self.double(self.double(x))




