import sys
sys.set_int_max_str_digits(0)
n: int = 1
__mamba_n_existed = "n" in locals()
__mamba_n_saved = n if __mamba_n_existed else None
match 10:
    case n: 
        print("in arm")

if __mamba_n_existed: 
    n = __mamba_n_saved
else: 
    del n
print(n)

