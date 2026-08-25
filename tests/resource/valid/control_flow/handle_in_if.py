def f() -> int:
    return 10

if True:
    try:
        x: int = f()
    except Exception as err:
        x: int = 3
else:
    try:
        x: int = f()
    except Exception as err:
        x: int = 3
