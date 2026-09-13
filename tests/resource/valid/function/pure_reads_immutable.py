class C: 
    x: int = 1
    def get(self) -> int: 
        return self.x



taylor: int = 7
def f(c: C) -> int: 
    return c.x + taylor


