from typing import TypeVar, Generic

class set(Generic[T]):
    def __init__(self): pass
    def __iter__(self) -> set_iterator[T]: pass

class set_iterator[T]:
    def __init__(self): pass
    def __next__(self) -> T: pass

class list(Generic[T]):
    def __init__(self): pass
    def __iter__(self) -> list_iterator[T]: pass

class list_iterator[T]:
    def __init__(self): pass
    def __next__(self) -> T: pass
