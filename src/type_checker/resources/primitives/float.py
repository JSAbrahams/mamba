class float:
    def __init__(self, arg: float): pass

    # def __add__(self, other: int) -> float: pass
    def __add__(self, other: float) -> float: pass
    # def __add__(self, other: complex) -> complex: pass
    # def __add__(self) -> float: return self

    # def __sub__(self, other: int) -> float: pass
    def __sub__(self, other: float) -> float: pass
    # def __sub__(self, other: complex) -> complex: pass
    # def __sub__(self) -> float: return -self

    # def __mul__(self, other: int) -> float: pass
    def __mul__(self, other: float) -> float: pass
    # def __mul__(self, other: complex) -> complex: pass

    def sqrt(self) -> float: pass

    # def __div__(self, other: int) -> float: pass
    def __div__(self, other: float) -> float: pass
    # def __div__(self, other: complex) -> complex: pass

    # def __floordiv__(self, other: int) -> float: pass
    def __floordiv__(self, other: float) -> float: pass

    # def __mod__(self, other: int) -> float: pass
    def __mod__(self, other: float) -> float: pass

    def __neg__(self) -> float: pass

    ## TODO add optional arguments and names arguments
    ## TODO re-add modulo
    # def __pow__(self, power: int) -> float: pass
    def __pow__(self, power: float) -> float: pass
    # def __pow__(self, power: complex) -> complex: pass

    # def __ge__(self, other: int) -> bool: pass
    def __ge__(self, other: float) -> bool: pass
    # def __gt__(self, other: int) -> bool: pass
    def __gt__(self, other: float) -> bool: pass

    # def __le__(self, other: int) -> bool: pass
    def __le__(self, other: float) -> bool: pass
    # def __lt__(self, other: int) -> bool: pass
    def __lt__(self, other: float) -> bool: pass

    def __bool__(self) -> bool: pass
    def __str__(self) -> str: pass
