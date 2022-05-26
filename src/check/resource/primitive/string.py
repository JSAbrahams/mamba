from typing import Union

class str:
    def __init__(self, arg: str) -> str: pass

    def __add__(self, other: Union[int, float, complex, bool, str]) -> str: pass
    def __str__(self) -> str: pass

    def __eq__(self, other: str) -> bool: pass

    def __iter__(self) -> str_iterator: pass

class str_iterator:
    def __init__(self): pass
    def __next__(self) -> str: pass

    def __bool__(self) -> bool: pass
    