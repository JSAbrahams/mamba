from typing import Callable, Optional

class X:
    def some_higher_order(self, fun: Callable[[int], int]) -> int:
        return fun(10)

    def fancy(self) -> Optional[int]:
        return self.some_higher_order(lambda x: x * 2)

x: X = X()
x.fancy()
