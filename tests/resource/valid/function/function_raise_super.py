from typing import Callable

class MyErr(Exception):
    def __init__(self):
        Exception.__init__(self)

class MyErr2(Exception):
    def __init__(self):
        Exception.__init__(self)

def f(fun: Callable[[int], int]):
    g()
def g():
    pass
