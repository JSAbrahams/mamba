class MyClass:
    x: int = None

    def __init__(self):
        match 10:
            case 2:
                self.x = 2
            case 3:
                self.x = 3
            case _:
                self.x = 3
