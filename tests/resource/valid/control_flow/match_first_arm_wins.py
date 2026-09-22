def classify(x: int) -> str:
    match x:
        case n if n > 0:
            return "positive"
        case 1:
            return "one"
        case _:
            return "other"

print(classify(1))
print(classify(0))

