from typing import Union

class complex:
    def __init__(self, n: Union[int, float], j: Union[int, float]) -> complex: pass

    def __add__(self, other: Union[int, float, complex]) -> complex: pass
    # def __add__(self): return self

    def __sub__(self, other: Union[int, float, complex]) -> complex: pass
    # def __sub__(self) -> complex: return -self

    def __mul__(self, other: Union[int, float, complex]) -> complex: pass

    def __div__(self, other: Union[int, float, complex]) -> complex: pass

    def __neg__(self) -> float: pass

    def __pow__(self, power: Union[int, float, complex], modulo=None) -> complex: pass

    def __str__(self) -> str: pass

    def __eq__(self, other: complex) -> bool:  pass
