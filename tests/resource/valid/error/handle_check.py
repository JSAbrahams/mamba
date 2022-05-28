MyErr1 = Exception("Something went wrong")


class MyErr2(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)


def f(x: int) -> int:
    if x < 0:
        raise MyErr1()
    else:
        if x > 10:
            raise MyErr2("Greater than 10.")
        else:
            return x + 2


a = None
try:
    a = f(10)
except MyErr1 as err:
    print("Something went wrong")
    a = -1
except MyErr2 as err:
    print("Something else went wrong")
    a = -2
