def g() -> int =>
    raise Exception("A")

def f(x: int) -> int =>
    try:
        return g()
    except:
        return x + 10
