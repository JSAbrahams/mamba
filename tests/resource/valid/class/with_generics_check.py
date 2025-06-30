class InnerClass:
    def inner(a: A) -> A:
        return a

class OuterClass:
    inner_class: InnerClass = InnerClass()

outer_class = OuterClass()
print(outer_class.inner_class.inner("hello world"))
