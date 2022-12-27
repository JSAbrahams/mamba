from typing import Union

class float(complex):
    def __init__(self, arg: Union[str, int, float]) -> float: pass

    # def __add__(self, other: complex) -> complex: pass
    # def __add__(self) -> float: return self
    def __add__(self, other: Union[int, float]) -> float: pass

    # def __sub__(self, other: complex) -> complex: pass
    # def __sub__(self) -> float: return -self
    def __sub__(self, other: Union[float, int]) -> float: pass

    # def __mul__(self, other: complex) -> complex: pass
    def __mul__(self, other: Union[int, float]) -> float: pass

    def sqrt(self) -> float: pass

    # def __truediv__(self, other: complex) -> complex: pass
    def __truediv__(self, other: Union[int, float]) -> float: pass

    def __floordiv__(self, other: Union[int, float]) -> float: pass

    def __mod__(self, other: Union[int, float]) -> float: pass

    def __neg__(self) -> float: pass

    ## TODO add optional arguments and names arguments
    ## TODO re-add modulo
    # def __pow__(self, power: complex) -> complex: pass
    def __pow__(self, power: Union[int, float]) -> float: pass

    def __ge__(self, other: Union[int, float]) -> bool: pass
    def __gt__(self, other: Union[int, float]) -> bool: pass
    def __le__(self, other: Union[int, float]) -> bool: pass
    def __lt__(self, other: Union[int, float]) -> bool: pass

    def __str__(self) -> str: pass

    def __eq__(self, other: float) -> bool:  pass
