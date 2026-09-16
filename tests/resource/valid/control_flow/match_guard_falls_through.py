def magnitude(x: int) -> str:
    match x:
        case n if n > 100:
            return "big"
        case n if n > 10:
            return "medium"
        case _:
            return "small"

print(magnitude(500))
print(magnitude(50))
print(magnitude(5))

