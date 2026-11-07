import sys
sys.set_int_max_str_digits(0)
def label(b: bool) -> str: 
    match b:
        case True: 
            return "yes"
        case False: 
            return "no"


print(label(True))
print(label(False))

