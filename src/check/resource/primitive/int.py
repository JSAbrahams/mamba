from typing import Union


class int(float):
    def __init__(self, arg: Union[str, float, int]) -> int: pass

    # def __add__(self, other: float) -> float: pass
    # def __add__(self, other: complex) -> complex: pass
    def __add__(self, other: int) -> int: pass

    # def __sub__(self, other: float) -> float: pass
    # def __sub__(self, other: complex) -> complex: pass
    def __sub__(self, other: int) -> int: pass

    # def __mul__(self, other: float) -> float: pass
    # def __mul__(self, other: complex) -> complex: pass
    def __mul__(self, other: int) -> int: pass

    def sqrt(self) -> float: pass

    # def __truediv__(self, other: float) -> float: pass
    # def __truediv__(self, other: complex) -> complex: pass
    def __truediv__(self, other: int) -> float: pass

    # def __floordiv__(self, other: float) -> float: pass
    def __floordiv__(self, other: int) -> int: pass

    # def __mod__(self, other: float) -> float: pass
    def __mod__(self, other: int) -> int: pass

    def __neg__(self) -> int:  pass

    # def __pow__(self, power: int, modulo=None) -> int: pass
    # def __pow__(self, power: float, modulo=None) -> float: pass
    # def __pow__(self, power: complex, modulo=None) -> complex: pass
    # def __pow__(self, power: float) -> float: pass
    # def __pow__(self, power: complex) -> complex: pass
    def __pow__(self, power: int) -> int: pass

    def __ge__(self, other: Union[int, float]) -> bool: pass

    def __gt__(self, other: Union[int, float]) -> bool: pass

    def __le__(self, other: Union[int, float]) -> bool: pass

    def __lt__(self, other: Union[int, float]) -> bool: pass

    def __str__(self) -> str: pass

    def __eq__(self, other: int) -> bool:  pass
