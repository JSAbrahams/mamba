class Wrapper: 
    def __init__(self, x: int): 
        self.x = x




def wrap(x: int) -> Wrapper: 
    return Wrapper(x)

print(wrap(8).x)

