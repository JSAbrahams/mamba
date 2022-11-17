def my_fun(a: int, b: int = 10, c: str = "Hello") -> str:
    return c + b + a

my_fun(1, 2, "hello world")
my_fun(1, 2)
my_fun(1)
