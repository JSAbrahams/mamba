def sign(x: int) -> str:
    match x:
        case n if n < 0:
            return "negative"
        case n if n > 0:
            return "positive"
        case _:
            return "zero"


print(sign(-3))
print(sign(3))
print(sign(0))
