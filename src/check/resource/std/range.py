from typing import List

class range:
    start: int = 0
    stop: int = 0
    step: int = 0

    def __iter__(self) -> range_iterator: pass
    def __str__(self) -> str: pass

class range_iterator:
    def __init__(self): pass
    def __next__(self) -> int: pass

class slice:
    indices: int = 0
    start: int = 0
    stop: int = 0
    step: int = 0

    def __str__(self) -> str: pass
