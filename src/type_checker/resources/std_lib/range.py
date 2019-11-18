class range:
    start: int = 0
    stop: int = 0
    step: int = 0

    def __iter__(self) -> range_iterator: pass

class range_iterator:
    def __init__(self): pass
    def __next__(self) -> int: pass