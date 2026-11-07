n: int = 1
if True:
    __mamba_n_existed = "n" in locals()
    __mamba_n_saved = n if __mamba_n_existed else None
    n: int = 2
    print(n)
    if __mamba_n_existed: 
        n = __mamba_n_saved
    else: 
        del n

if True:
    __mamba_n_existed = "n" in locals()
    __mamba_n_saved = n if __mamba_n_existed else None
    n: str = "s"
    print(n)
    if __mamba_n_existed: 
        n = __mamba_n_saved
    else: 
        del n

print(n)

