from typing import Tuple

b: set[Tuple[int, int]] = {(1, 4), (2, 5)}
for (first, second) in b:
    print(first + second)
