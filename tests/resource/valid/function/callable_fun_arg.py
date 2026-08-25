from typing import Callable


def f(x: Callable[[int], int], y: int) -> int:
    return x(y)


f(lambda z: z + 2, 4)
