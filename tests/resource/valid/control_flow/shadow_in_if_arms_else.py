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
    x: MyClass1 = MyClass1()
    x.f1()
