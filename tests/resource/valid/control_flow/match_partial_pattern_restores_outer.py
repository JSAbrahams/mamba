n: int = 1
__mamba_n_existed = "n" in locals()
__mamba_n_saved = n if __mamba_n_existed else None
match (5, 1):
    case (n, 0): 
        print("never")
    case _: 
        print("fallback")

if __mamba_n_existed: 
    n = __mamba_n_saved
else: 
    del n
print(n)

