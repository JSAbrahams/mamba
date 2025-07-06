class MyErr(Exception):
    def __init__(self):
        Exception.__init__(self)

def f(x: int):
    print("nothing")

def g(x: int):
    f(x)
