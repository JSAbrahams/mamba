class Err(Exception):
    def __init__(self, msg: str):
        super().__init__(msg)

def f(x: int) -> int:
    if x > 0:
        raise Err("asdf")
    else:
        return 10

f(10)
f(10)
