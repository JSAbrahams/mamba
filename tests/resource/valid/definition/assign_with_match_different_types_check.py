from typing import Union
class MyClass:
    pass

class MyClass1:
    pass

class MyClass2:
    pass

match 40:
    case 2:
        a: Union[MyClass, MyClass1, MyClass2] = MyClass()
    case 4:
        a: Union[MyClass, MyClass1, MyClass2] = MyClass1()
    case _:
        a: Union[MyClass, MyClass1, MyClass2] = MyClass2()
