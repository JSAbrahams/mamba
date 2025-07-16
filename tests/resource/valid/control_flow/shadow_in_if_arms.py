class MyClass1:
    def f1(self):
        print("1")

class MyClass2:
    def f2(self):
        print("2")

if True:
    x: MyClass1 = MyClass1()
    x.f1()
else:
    x: MyClass2 = MyClass2()
    x.f2()
