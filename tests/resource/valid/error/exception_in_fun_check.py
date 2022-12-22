def g() -> int =>
    raise Exception("A")

def f(x: int) -> int:
    try:
        g()
    except Exception as err:
        x + 10
