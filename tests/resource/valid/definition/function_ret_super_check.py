from typing import Callable, Optional

def some_higher_order(fun: Callable[[int], int]) -> int:
    fun(10)

def fancy() -> Optional[int]:
    some_higher_order(lambda x: x * 2)
