class B: 
    my_num: int = 10


class A: 
    def __init__(self, b: B): 
        self.b = b




a: A = A(B())
a.b.my_num = 20

