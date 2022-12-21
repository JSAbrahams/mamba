def g() -> int =>
    raise Exception("A")

def f(x: int) -> int =>
    try:
        return g()
    except Exception as err:
        return x + 10
