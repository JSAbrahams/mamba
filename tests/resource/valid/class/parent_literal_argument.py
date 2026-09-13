class Base: 
    def __init__(self, code: int, offset: int): 
        self.code = code
        self.offset = offset




class Derived(Base): 
    def __init__(self): 
        Base.__init__(self, 404, 1 + 1)




d: Derived = Derived()
print(d.code)
print(d.offset)

