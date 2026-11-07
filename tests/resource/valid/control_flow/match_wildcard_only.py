import sys
sys.set_int_max_str_digits(0)
match "anything":
    case _: 
        a: str = "always"

print(a)

