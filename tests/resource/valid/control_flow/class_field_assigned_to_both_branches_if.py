class MyClass:
    x: int = None

    def __init__(self):
        if False:
            self.x = 10
        else:
            self.x = 20
