from typing import Callable


class IntTransform(Callable[[int], int]):
    def __init__(self):
        Callable.__init__(self)
