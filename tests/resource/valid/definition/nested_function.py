class A:
    b: B

class B:
    def my_c(self) -> C:
        C()

class C:
    my_field: Int = 10

a = A()
print(a.b.my_c().my_field)
