from typing import Generic


class collection(Generic[T]):
    def __init__(self): pass

    def __iter__(self) -> collection_iter[T]: pass

    def __contains__(self, item: T) -> bool: pass


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

    def __getitem__(self, item: Union[int, slice]) -> T: pass

    def __bool__(self) -> bool: pass

    def __str__(self) -> str: pass


class list_iterator(Generic[T]):
    def __init__(self): pass

    def __next__(self) -> T: pass


class Tuple(Generic[T], collection[T]):
    def __init__(self): pass

    def __iter__(self) -> tuple_iterator[T]: pass

    def __getitem__(self, item: Union[int, slice]) -> T: pass

    def __str__(self) -> str: pass


class tuple_iterator(Generic[T]):
    def __init__(self): pass

    def __next__(self) -> T: pass


class dict(Generic[T, R]):
    def __init__(self): pass

    def __getitem__(self, item: T) -> R: pass

    def __iter__(self) -> dictkeyiterator[T]: pass

    def __contains__(self, item: T) -> bool: pass

    def keys(self) -> dict_keys[T]: pass

    def values(self) -> dict_values[R]: pass


class dict_keys(Generic[T]):
    def __init__(self): pass

    def __iter__(self) -> dict_keyiterator[T]: pass

    def __contains__(self, item: T) -> bool: pass


class dict_values(Generic[T]):
    def __init__(self): pass

    def __iter__(self) -> dict_valueiterator[T]: pass

    def __contains__(self, item: T) -> bool: pass


class dict_keyiterator(Generic[T]):
    def __init__(self): pass

    def __next__(self) -> T: pass


class dict_valueiterator(Generic[T]):
    def __init__(self): pass

    def __next__(self) -> T: pass
