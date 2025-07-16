class MyErr1(Exception):
    def __init__(self):
        Exception.__init__(self, "Something went wrong")


class MyErr2(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)


def f(x: int) -> int:
    return x


a: int = None
try:
    a: int = f(10)
except MyErr1:
    print("Something went wrong")
    a = -1
except MyErr2:
    print("Something else went wrong")
    a = -2
