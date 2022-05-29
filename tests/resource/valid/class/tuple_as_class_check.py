from typing import Tuple


class MyTuple(Tuple[int, int, str]):
    def __init__(self):
        Tuple.__init__(self)
