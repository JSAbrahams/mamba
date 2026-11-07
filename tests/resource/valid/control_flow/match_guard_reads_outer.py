limit: int = 10
def under(x: int, bound: int) -> str:
    match x:
        case n if n < bound:
            return "under"
        case _:
            return "over"

print(under(5, limit))
print(under(50, limit))

