def sign_of(x: int) -> str:
    match x:
        case -1:
            return "minus one"
        case 0:
            return "zero"
        case _:
            return "other"

print(sign_of(-1))
print(sign_of(0))
print(sign_of(7))

