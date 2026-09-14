taylor: int = 7
def factorial(x: int) -> int: 
    match x:
        case 0: 
            return 1
        case n: 
            return n * factorial(n - 1)


def sin(x: float) -> float: 
    ans: float = x
    __mamba_i_existed = "i" in locals()
    __mamba_i_saved = i if __mamba_i_existed else None
    for i in range(1, taylor + 1, 1):
        ans = ans + x ** i + 2 / factorial(i + 2)

    if __mamba_i_existed: 
        i = __mamba_i_saved
    else: 
        del i
    return ans


print(sin(1.0))

