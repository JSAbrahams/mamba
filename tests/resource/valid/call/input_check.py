num: str = input("Compute factorial: ")

if num.is_digit():
    result: int = int(num)
    print(f"Factorial {num} is: {result}.")
else:
    print("Input was not an integer.")
