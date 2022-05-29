from typing import NewType

MyType = NewType('MyType', str)

a: MyType = "MyString"
print(a)
