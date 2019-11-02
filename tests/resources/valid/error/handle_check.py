class MyErr1(Exception):

class MyErr2(Exception):

# TODO fix fragile parsing rules for if statements
def f(x: int) -> int: return MyErr1() if x < 0 else MyErr2 if x > 10 else x + 2

try:
    a = f(10)
except MyErr1 as err:
    print('Something went wrong')
except MyErr2 as err:
    print('Something else went wrong')