__mamba_i_existed = "i" in locals()
__mamba_i_saved = i if __mamba_i_existed else None
for i in range(0, 2 + 1, 1):
    match i:
        case 0:
            print("zero")
        case 1:
            print("one")
        case _:
            print("many")

if __mamba_i_existed:
    i = __mamba_i_saved
else:
    del i

