def g() -> int:
    raise Exception("A")


a: int = None
try:
    a: int = g()
except Exception as err:
    a = 10
