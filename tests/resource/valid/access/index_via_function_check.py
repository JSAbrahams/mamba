def f() -> slice:
    slice(0, 2 - 1, 3)
def g() -> slice:
    slice(0, 2, 3)

def i() -> range:
    range(0, 2, 3)
def j() -> range:
    range(0, 2 + 1, 3)

x = [1, 2, 3]
x[f()]
