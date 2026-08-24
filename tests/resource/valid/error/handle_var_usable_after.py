class MyErr(Exception):
    def __init__(self, message: str):
        Exception.__init__(self)
        self.message = message


def function_may_throw_err() -> int:
    return 10


a: int = None
try:
    a: int = function_may_throw_err()
except MyErr as err:
    print(f"We have a problem: {err.message}.")
    a = 20

print(f"a has value {a}.")
