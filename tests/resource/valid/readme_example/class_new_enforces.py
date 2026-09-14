class FractionErr(Exception): 
    def __init__(self, message: str): 
        Exception.__init__(self, message)




class Fraction: 
    def __init__(self, num: int, den: int): 
        self.num = num
        self.den = den


    def new(num: int, den: int) -> Fraction: 
        if den == 0: 
            raise FractionErr("Denominator is zero")
        else: 
            return Fraction(num, den)



half: Fraction = Fraction.new(1, 2)
print(half.den)

