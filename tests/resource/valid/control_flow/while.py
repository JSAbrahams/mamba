import sys
sys.set_int_max_str_digits(0)
b: bool = False
e: str = "Hello world"
while b:
    print(b)

d: bool = True
while d:
    print(d and False)

    print(d and True)

print("hello")
