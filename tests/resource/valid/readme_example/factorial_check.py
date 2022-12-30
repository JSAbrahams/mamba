def factorial(x: int) -> int:
    match x:
        case 0:
            return 1
        case n:
            return n * factorial(n - 1)

num: str = input("Compute factorial: ")

if num.is_digit():
    result = factorial(int(num))
    print(f"Factorial {num} is: {result}.")
else:
    print("Input was not an integer.")
