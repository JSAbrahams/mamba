class Err(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)

def f(x: int) -> int:
    if x > 0:
        return 10
    else:
        raise Err("Expected positive number.")

def g() -> int:
    raise Err("Error always raised")

def h(x: int) -> int:
    if x < 0:
        raise Err("Less than")
    else:
        raise Err("Greater Than")

f(10)
