from typing import Callable
from typing import Optional

class X:
    def some_higher_order(self, fun: Callable[[int], int]) -> int:
        fun(10)

    def fancy(self) -> Optional[int]:
        self.some_higher_order(lambda x: x * 2)

x: X = X()
x.fancy()
