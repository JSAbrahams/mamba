class int:
    def __add__(self, other: int) -> int: pass
    def __add__(self, other: float) -> float: pass
    def __add__(self, other: complex) -> complex: pass

    def __sub__(self, other: int) -> int: pass
    def __sub__(self, other: float) -> float: pass
    def __sub__(self, other: complex) -> complex: pass

    def __mul__(self, other: int) -> int: pass
    def __mul__(self, other: float) -> float: pass
    def __mul__(self, other: complex) -> complex: pass

    def __div__(self, other: int) -> float: pass
    def __div__(self, other: float) -> float: pass
    def __div__(self, other: complex) -> complex: pass

    def __floordiv__(self, other: int) -> int: pass
    def __floordiv__(self, other: float) -> float: pass

    def __mod__(self, other: int) -> int: pass
    def __mod__(self, other: float) -> float: pass

    def __neg__(self) -> int:  pass

    def __pow__(self, power: int, modulo=None) -> int: pass
    def __pow__(self, power: float, modulo=None) -> float: pass
    def __pow__(self, power: complex, modulo=None) -> complex: pass
