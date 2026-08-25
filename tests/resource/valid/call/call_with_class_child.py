class MyClass:
    pass

class MyChildClass(MyClass):
    def __init__(self):
        MyClass.__init__(self)

def my_fun(arg: MyClass):
    pass

my_fun(MyClass())
my_fun(MyChildClass())
