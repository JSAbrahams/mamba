import sys
sys.set_int_max_str_digits(0)
def f() -> range:
    return range(0, 2, 1)

__mamba_x_existed = "x" in locals()
__mamba_x_saved = x if __mamba_x_existed else None
for x in f():
    print(x)

if __mamba_x_existed:
    x = __mamba_x_saved
else:
    del x
