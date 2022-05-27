a: bool = True
b = a

c: bool = False or True and False
d: bool = c and b and a
e: bool = not False

a and b
c or d
not e

f: bool = True
g: bool = False
h: bool = True
i: bool = True
j: bool = False

c is d
e is not f
isinstance(g, h)
not isinstance(i, j)