num: str = input("Compute factorial: ")

if num.is_digit():
    __mamba_result_existed = "result" in locals()
    __mamba_result_saved = result if __mamba_result_existed else None
    result: int = int(num)
    print(f"Factorial {num} is: {result}.")
    if __mamba_result_existed:
        result = __mamba_result_saved
    else:
        del result
else:
    print("Input was not an integer.")
