class bool:
    def __init__(self, arg: Union[int, str, bool]) -> bool: pass

    def __bool__(self) -> bool: pass

    def __str__(self) -> str: pass

    def __eq__(self, other: bool) -> bool:  pass
