import math
from typing import Tuple


class Deps2:
    gt: bool = None
    leq: bool = None
    sq: float = None
    tup: Tuple[int, int] = None
    st: set[int] = None
    dc: dict[int, int] = None
    ifv: int = None
    later: int = None

    def __init__(self, x: int):
        self.x = x
        self.later = self.x * 2
        self.gt = self.x > self.later
        self.leq = self.x <= self.later
        self.sq = math.sqrt(self.later)
        self.tup = (self.x, self.later)
        self.st = {self.x, self.later}
        self.dc = {self.x: self.later}
        self.ifv = self.x if self.gt else self.later


d: Deps2 = Deps2(3)
print(d.gt)
