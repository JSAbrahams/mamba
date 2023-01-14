from typing import Any, Tuple
a: set[Any] = {}
b: set[int] = { 10, 20 }
c: set[set[Any]] = { {}, {} }
d: set[set[str]] = { { "a" }, { "c", "d" } }
e: set[Tuple[int, int]] = { (x, 0) for x in b }
f: set[int] = { x ** 2 for x in range(0, 10 + 1, 1) }
