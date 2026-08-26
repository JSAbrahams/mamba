class MyClass1:
    def f1(self):
        pass

class MyClass2:
    def f2(self):
        pass

x: MyClass2 = MyClass2()
if True:
    x.f2()
else:
    __mamba_x_existed = "x" in locals()
    __mamba_x_saved = x if __mamba_x_existed else None
    x: MyClass1 = MyClass1()
    x.f1()
    if __mamba_x_existed:
        x = __mamba_x_saved
    else:
        del x
