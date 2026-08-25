def f(x: int) -> str:
    match x:
        case 1:
            return "One"
        case 2:
            return "Two"
        case _:
            return "Three"
