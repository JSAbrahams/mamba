class MyClass1:
    def f1(self):
        print("1")

class MyClass2:
    def f2(self):
        print("2")

if True:
    __mamba_x_existed = "x" in locals()
    __mamba_x_saved = x if __mamba_x_existed else None
    x: MyClass1 = MyClass1()
    x.f1()
    if __mamba_x_existed:
        x = __mamba_x_saved
    else:
        del x
else:
    __mamba_x_existed = "x" in locals()
    __mamba_x_saved = x if __mamba_x_existed else None
    x: MyClass2 = MyClass2()
    x.f2()
    if __mamba_x_existed:
        x = __mamba_x_saved
    else:
        del x
