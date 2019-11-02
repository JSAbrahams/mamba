b = {1,2}
for first in b:
    print(first)

b = {(1, 4), (2, 5)}
for (first, second) in b:
    print(first + second)

e = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
for d in e:
    print(d)
    print(d - 1)

    print(d + 1)

for i in range(0, 34, 1):
    print(i)

for i in range(0, 345 + 1, 1):
    print(i)

a = 1
b = 112
for i in range(a, b, 1):
    print('hello')

c = 2451
for i in range(a, c + 1, 20):
    print('world')

for i in ([1,2], {3,4}):
    print(i)
