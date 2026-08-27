class Deps:
    cmp: bool = None
    eqcmp: bool = None
    geqcmp: bool = None
    logic: bool = None
    logic2: bool = None
    notv: bool = None
    negv: int = None
    posv: int = None
    subv: int = None
    modv: int = None
    powv: int = None
    divv: float = None
    fdivv: int = None
    inv: bool = None
    later: int = None

    def __init__(self, x: int):
        self.x = x
        self.later = self.x * 2
        self.cmp = self.x < self.later
        self.eqcmp = self.x == self.later
        self.geqcmp = self.x >= self.later
        self.logic = self.cmp or self.eqcmp
        self.logic2 = self.cmp and self.eqcmp
        self.notv = not self.cmp
        self.negv = -self.later
        self.posv = +self.later
        self.subv = self.x - self.later
        self.modv = self.x % self.later
        self.powv = self.x ** self.later
        self.divv = self.x / self.later
        self.fdivv = self.x // self.later
        self.inv = self.x in [self.later]


d: Deps = Deps(3)
print(d.cmp)
