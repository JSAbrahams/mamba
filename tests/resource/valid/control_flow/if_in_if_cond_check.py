a: bool = True
b: bool = True
c: bool = False

d: bool = b if a else c
e: bool = b if d else c
