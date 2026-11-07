flag: bool = True
def check(x: int) -> str:
    match x:
        case 0 if flag:
            return "zero and flagged"
        case 0:
            return "zero"
        case _:
            return "other"

print(check(0))
print(check(1))

