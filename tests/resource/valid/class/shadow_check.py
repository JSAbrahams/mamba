class MyClass1:
    def f1(self):
        print("1")

class MyClass2:
    def f2(self):
        print("2")

class MyClass3:
    def f3(self):
        print("3")

class MyClass4:
    def f4(self):
        print("4")

x: MyClass1 = MyClass1()
x.f1()

x: MyClass2 = MyClass2()
x.f2()

class MyClass:
    x: MyClass3 = MyClass3()

    def g():
        x.f3()

    def f(x: MyClass4):
        x.f4()
