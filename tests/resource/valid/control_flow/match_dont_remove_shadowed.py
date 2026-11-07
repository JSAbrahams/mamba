n: int = 10
__mamba_n_existed = "n" in locals()
__mamba_n_saved = n if __mamba_n_existed else None
match 10:
    case n: 
        print("10")

if __mamba_n_existed: 
    n = __mamba_n_saved
else: 
    del n
print(n)

