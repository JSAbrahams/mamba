class MyException(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)

def g() -> int:
    raise MyException("A")

def f(x: int) -> int:
    try:
        return g()
    except Exception as err:
        return x + 10
