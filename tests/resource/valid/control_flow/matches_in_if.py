if True:
    match 10:
        case 2:
            x: int = 3
        case _:
            x: int = 4
else:
    match 20:
        case _:
            x: int = 2
