import sys
sys.set_int_max_str_digits(0)
def fun_a(): print("hello world")
def fun_b(x: int): print(f"hello {x}")

fun_a()
fun_b(123)
