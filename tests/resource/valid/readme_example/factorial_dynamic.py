def factorial(x: int) -> int: 
    match x:
        case 0: 
            return 1
        case n: 
            __mamba_ans_existed = "ans" in locals()
            __mamba_ans_saved = ans if __mamba_ans_existed else None
            ans: int = 1
            __mamba_i_existed = "i" in locals()
            __mamba_i_saved = i if __mamba_i_existed else None
            for i in range(1, n + 1, 1):
                ans = ans * i

            if __mamba_i_existed: 
                i = __mamba_i_saved
            else: 
                del i
            return ans



print(factorial(5))

