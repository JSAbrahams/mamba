from typing import Any
c: set[int] = {10, 20}
d: set[int] = {3}
c_squared: set[int] = {x * x for x in c}
empty_set: set[Any] = {}
e: dict[str, int] = {"do": 1, "ree": 2, "meee": 3}
print(e["ree"])

