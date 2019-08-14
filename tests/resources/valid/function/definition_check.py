class Something:
    def __add__(self, other): a + b
    def __sub__(self, other): a - b
    # for now no square root
    def __mul__(self, other): a * b
    def __truediv__(self, other): a/b
    def __floordiv__(self, other): a // b
    def __pow__(self, other): a ** b
    def __mod__(self, other): a % b
    def __eq__(self, other): a == b
    def __gt__(self, other): a > b
    def __lt__(self, other): a < b

def fun_a():
    print(c)
    if a and b: print(c)
    if c or d: print(e)

def fun_b(b): print(c)

def fun_c(d): print(g)

def fun_d(h): print(l)

def fun_e(m, o, r): print(u)

def fun_v(w, y, ab): print(u)

def factorial(x): x * factorial(x - 1)

def call_higher_order(): some_higher_order(lambda x : x * 2)
