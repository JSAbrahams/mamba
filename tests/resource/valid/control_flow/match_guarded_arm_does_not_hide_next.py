def f(x: int) -> str: 
    match x:
        case 0 if x > 5: 
            return "never"
        case 0: 
            return "zero"
        case _: 
            return "other"


print(f(0))
print(f(1))

