def band(x: int) -> str:
    match x:
        case n if n > 0 and n < 10:
            return "single digit"
        case n if n >= 10 or n < 0:
            return "out of range"
        case _:
            return "zero"

print(band(5))
print(band(50))
print(band(0))

