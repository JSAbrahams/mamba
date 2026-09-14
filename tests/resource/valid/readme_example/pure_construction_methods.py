class Counter: 
    count: int = None
    def __init__(self, start: int): 
        self.start = start
        self.count = self.start


    def tick(self): 
        self.count = self.count + 1
        print(self.count)




c: Counter = Counter(1)
c.tick()

