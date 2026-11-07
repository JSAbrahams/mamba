a: str = "d"

b, bb, bbb = (0, 1, 2)

match (b, bb, bbb):
    case (0, 1, 2):
        print("hello world")
    case _:
        print("something else")

nested: str = "other"

match nested:
    case "a":
        "b"
        "c"
    case "c":
        match nested:
            case "other":
                "even_other"
            case "other_one":
                "better_one"
            case _:
                "default"
    case _:
        "neither"
