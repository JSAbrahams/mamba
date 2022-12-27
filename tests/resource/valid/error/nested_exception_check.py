class MyException1(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)

class MyException2(Exception):
    def __init__(self, msg: str):
        Exception.__init__(self, msg)

def f(x: int) -> int:
    match x:
        case 0:
            return 20
        case 1:
            raise MyException1()
        case 2:
            raise MyException2()

def g() -> int:
    try:
        f(2)
    except Exception as err:
        print("a")

g()
