from typing import Union

class MyClass:
    def __str__() -> str:
        return "M"

a: Union[int, MyClass] = 20 if True else MyClass()

print(a)
