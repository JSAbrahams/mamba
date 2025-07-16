if True:
    match 10:
        case 2:
            x = 3
        case _:
            x = 4
else:
    match 20:
        case _:
            x = 2
