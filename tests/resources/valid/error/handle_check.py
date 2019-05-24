a = None

try:
    a = f(10)
except MyErr1 as err:
    print('Something went wrong')
except MyErr2 as err:
    print('Something else went wrong')