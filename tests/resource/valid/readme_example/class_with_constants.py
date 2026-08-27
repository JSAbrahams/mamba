class Point2D: 
    x: int = None
    y: int = None
    def __init__(self, ORIGIN_X: int, ORIGIN_Y: int): 
        self.ORIGIN_X = ORIGIN_X
        self.ORIGIN_Y = ORIGIN_Y
        self.x = self.ORIGIN_X
        self.y = self.ORIGIN_Y


    def move(self, dx: int, dy: int): 
        self.x = self.x + dx
        self.y = self.y + dy


    def reset(self): 
        self.x = self.ORIGIN_X
        self.y = self.ORIGIN_Y


    def info(self) -> str: 
        return f"Currently at ({self.x}, {self.y}), originally from ({self.ORIGIN_X}, {self.ORIGIN_Y})"




