class MyErr(Exception):
    def __init__(self, message: str):
        Exception.__init__(self)
        self.message = message


class MyOtherErr(Exception):
    def __init__(self, message: str):
        Exception.__init__(self)
        self.message = message


def function_may_throw_err() -> int:
    return 10


def g():
    a: int = None
    try:
        a: int = function_may_throw_err()
    except MyErr as err:
        print(f"We have a problem: {err.message}.")
        return None
    except MyOtherErr as err:
        print(f"We have another problem: {err.message}.")
        a = 0

    print(f"a has value {a}.")


g()
