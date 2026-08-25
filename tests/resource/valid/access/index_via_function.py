def f() -> slice:
    return slice(0, 2 - 1, 1)
def g() -> slice:
    return slice(0, 2, 1)

def i() -> range:
    return range(0, 2, 1)
def j() -> range:
    return range(0, 2 + 1, 1)

x: list[int] = [1, 2, 3]
x[f()]
