class MyType(str):
    def __init__(self):
        str.__init__(self)

a: MyType = "my_string"
print(a)
