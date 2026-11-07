import sys
sys.set_int_max_str_digits(0)
message: str = "may be mutable, for now"

class MyClass:
    def f(self):
        print(message)
