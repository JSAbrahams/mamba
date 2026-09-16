# `NoneType` is what CPython calls the type of `None`, and `None` itself is a keyword, so it
# cannot name a class here. `python_to_concrete` maps it back to Mamba's `None`.
class NoneType:
    def __init__(self): pass

    def __bool__(self) -> bool: pass

    def __str__(self) -> str: pass
