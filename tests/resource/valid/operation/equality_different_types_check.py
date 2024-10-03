class MyClass:
    def __eq__(self, other: MyOtherClass) -> bool:
        return True

    def __ne__(self, other: MyOtherClass) -> bool:
        return False


class MyOtherClass:
    pass


a: MyClass = MyClass()
b: MyOtherClass = MyOtherClass()

a == b
a != b
