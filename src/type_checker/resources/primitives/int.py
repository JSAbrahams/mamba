class int:
    def __init__(self, arg: int): pass

    def __add__(self, other: int) -> int: pass
    def __add__(self, other: float) -> float: pass
    def __add__(self, other: complex) -> complex: pass
    def __add__(self) -> int: return self

    def __sub__(self, other: int) -> int: pass
    def __sub__(self, other: float) -> float: pass
    def __sub__(self, other: complex) -> complex: pass
    def __sub__(self) -> int: return -self

    def __mul__(self, other: int) -> int: pass
    def __mul__(self, other: float) -> float: pass
    def __mul__(self, other: complex) -> complex: pass

    def sqrt(self) -> float: pass

    def __div__(self, other: int) -> float: pass
    def __div__(self, other: float) -> float: pass
    def __div__(self, other: complex) -> complex: pass

    def __floordiv__(self, other: int) -> int: pass
    def __floordiv__(self, other: float) -> float: pass

    def __mod__(self, other: int) -> int: pass
    def __mod__(self, other: float) -> float: pass

    def __neg__(self) -> int:  pass

    # def __pow__(self, power: int, modulo=None) -> int: pass
    # def __pow__(self, power: float, modulo=None) -> float: pass
    # def __pow__(self, power: complex, modulo=None) -> complex: pass
    def __pow__(self, power: int) -> int: pass
    def __pow__(self, power: float) -> float: pass
    def __pow__(self, power: complex) -> complex: pass

    def __ge__(self, other: int) -> bool: pass
    def __ge__(self, other: float) -> bool: pass
    def __gt__(self, other: int) -> bool: pass
    def __gt__(self, other: float) -> bool: pass

    def __le__(self, other: int) -> bool: pass
    def __le__(self, other: float) -> bool: pass
    def __lt__(self, other: int) -> bool: pass
    def __lt__(self, other: float) -> bool: pass

    def __bool__(self) -> bool: pass
    def __str__(self) -> str: pass
