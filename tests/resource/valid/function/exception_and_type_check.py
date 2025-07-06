class MyErr(Exception):
    def __init__(self):
        Exception.__init__(self)

def f(x: int) -> int:
    raise MyErr()
