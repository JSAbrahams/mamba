match 2:
    case 2: 
        __mamba_doubled_existed = "doubled" in locals()
        __mamba_doubled_saved = doubled if __mamba_doubled_existed else None
        doubled: int = 2 * 2
        a: int = doubled
        if __mamba_doubled_existed: 
            doubled = __mamba_doubled_saved
        else: 
            del doubled

    case _: 
        a: int = 0

print(a)

