def f():
    slice(0, 2, 3)
def g():
    slice(0, 2 - 1, 3)

def i():
    range(0, 2 - 1, 3)
def j():
    range(0, 2, 3)

x = [1, 2, 3]
x[f()]
