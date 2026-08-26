b: set[int] = {1, 2}
__mamba_b_existed = "b" in locals()
__mamba_b_saved = b if __mamba_b_existed else None
__mamba_new_existed = "new" in locals()
__mamba_new_saved = new if __mamba_new_existed else None
for b in b:
    print(b + 5)
    new: int = b + 1
    new = 30
    print(new)

if __mamba_b_existed:
    b = __mamba_b_saved
else:
    del b
if __mamba_new_existed:
    new = __mamba_new_saved
else:
    del new

e: set[int] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
__mamba_d_existed = "d" in locals()
__mamba_d_saved = d if __mamba_d_existed else None
for d in e:
    print(d)
    print(d - 1)
    print(d + 1)

if __mamba_d_existed:
    d = __mamba_d_saved
else:
    del d

__mamba_i_existed = "i" in locals()
__mamba_i_saved = i if __mamba_i_existed else None
for i in range(0, 34, 1):
    print(i)

if __mamba_i_existed:
    i = __mamba_i_saved
else:
    del i

__mamba_i_existed = "i" in locals()
__mamba_i_saved = i if __mamba_i_existed else None
for i in range(0, 345 + 1, 1):
    print(i)

if __mamba_i_existed:
    i = __mamba_i_saved
else:
    del i

a: int = 1
b: int = 112
__mamba_i_existed = "i" in locals()
__mamba_i_saved = i if __mamba_i_existed else None
for i in range(a, b, 1):
    print("hello")

if __mamba_i_existed:
    i = __mamba_i_saved
else:
    del i

c: int = 2451
__mamba_i_existed = "i" in locals()
__mamba_i_saved = i if __mamba_i_existed else None
for i in range(a, c + 1, 20):
    print("world")

if __mamba_i_existed:
    i = __mamba_i_saved
else:
    del i
