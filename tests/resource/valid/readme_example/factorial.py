def factorial(x: int) -> int: 
    match x:
        case 0: 
            return 1
        case n: 
            return n * factorial(n - 1)


num: int = 5
if num >= 0: 
    __mamba_result_existed = "result" in locals()
    __mamba_result_saved = result if __mamba_result_existed else None
    result: int = factorial(num)
    print(f"Factorial {num} is: {result}.")
    if __mamba_result_existed: 
        result = __mamba_result_saved
    else: 
        del result

else: 
    print("Factorial is undefined for negative integers.")

