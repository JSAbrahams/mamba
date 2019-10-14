class str:
    def __add__(self, other: int) -> string: pass
    def __add__(self, other: float) -> string: pass
    def __add__(self, other: complex) -> string: pass
    def __add__(self, other: bool) -> string: pass
    def __add__(self, other: string) -> string: pass

    def __iter__(self) -> str_iterator: pass

class str_iterator:
    def __init__(self): pass
    def __next__(self) -> char: pass
