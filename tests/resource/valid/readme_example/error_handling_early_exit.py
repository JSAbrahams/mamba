class MyErr(Exception): 
    def __init__(self, message: str): 
        Exception.__init__(self, message)




class MyOtherErr(Exception): 
    def __init__(self, message: str): 
        Exception.__init__(self, message)




def function_may_throw_err() -> int: 
    return 10

a: int = function_may_throw_err()
print(f"a has value {a}.")

