class complex:
    def __add__(self, other: int) -> complex: pass
    def __add__(self, other: float) -> complex: pass
    def __add__(self, other: complex) -> complex: pass
    def __add__(self): return self

    def __sub__(self, other: int) -> complex: pass
    def __sub__(self, other: float) -> complex: pass
    def __sub__(self, other: complex) -> complex: pass
    def __sub__(self) -> complex: return -self

    def __mul__(self, other: int) -> complex: pass
    def __mul__(self, other: float) -> complex: pass
    def __mul__(self, other: complex) -> complex: pass

    def __div__(self, other: int) -> complex: pass
    def __div__(self, other: float) -> complex: pass
    def __div__(self, other: complex) -> complex: pass

    def __neg__(self) -> float: pass

    def __pow__(self, power: int, modulo=None) -> float: pass
    def __pow__(self, power: float, modulo=None) -> float: pass
    def __pow__(self, power: complex, modulo=None) -> complex: pass
