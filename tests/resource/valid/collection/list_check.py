from typing import Any
a: list[Any] = [ ]
g: list[int] = [ 1, 2 ]
h: list[int] = [ 4, 9 * 9 % 3 ]
i: list[int] = [ x for x in h if x > 0 and x > 3 ]
j: list[int] = [ (x, 0) for x in h if x > 0 ]
k: list[int] = [ x ** 2 for x in range(0, 10 + 1, 1) ]
