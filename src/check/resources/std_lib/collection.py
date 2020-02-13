from typing import TypeVar, Generic, Union

class set(Generic[T]):
    def __init__(self): pass
    def __iter__(self) -> set_iterator[T]: pass

    def __bool__(self) -> bool: pass
    def __str__(self) -> str: pass

class set_iterator(Generic[T]):
    def __init__(self): pass
    def __next__(self) -> T: pass

class list(Generic[T]):
    def __init__(self): pass
    def __iter__(self) -> list_iterator[T]: pass

    def __bool__(self) -> bool: pass
    def __str__(self) -> str: pass

class list_iterator(Generic[T]):
    def __init__(self): pass
    def __next__(self) -> T: pass
