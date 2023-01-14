from typing import Tuple
x: list[int] = [1, 2, 3]
y: list[str] = ["a", "b", "c"]
xy: set[Tuple[int, str]] = {{(l, m) for l in x if l > 0} for m in y if m != "c"}
