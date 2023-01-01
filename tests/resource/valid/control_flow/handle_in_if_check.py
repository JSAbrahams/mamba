def f() -> int:
    return 10

if True:
    try:
        x = f()
    except Exception as err:
        x = 3
else:
    try:
        x = f()
    except Exception as err:
        x = 3
