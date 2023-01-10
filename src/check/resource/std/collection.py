from typing import TypeVar, Generic, Union

class collection(Generic[T]):
    def __init__(self): pass
    def __iter__(self) -> collection_iter[T]: pass

class collection_iter(Generic[T]):
    def __init__(self): pass
    def __next__(self) -> T: pass

class set(Generic[T], collection[T]):
    def __init__(self): pass
    def __iter__(self) -> set_iterator[T]: pass

    def __bool__(self) -> bool: pass
    def __str__(self) -> str: pass

class set_iterator(Generic[T]):
    def __init__(self): pass
    def __next__(self) -> T: pass

class list(Generic[T], collection[T]):
    def __init__(self): pass
    def __iter__(self) -> list_iterator[T]: pass

    def __bool__(self) -> bool: pass
    def __str__(self) -> str: pass

class list_iterator(Generic[T]):
    def __init__(self): pass
    def __next__(self) -> T: pass

class Tuple(Generic[T], collection[T]):
    def __init__(self): pass
    def __iter__(self) -> tuple_iterator[T]: pass

    def __str__(self) -> str: pass

class tuple_iterator(Generic[T]):
    def __init__(self): pass
    def __next__(self) -> T: pass
