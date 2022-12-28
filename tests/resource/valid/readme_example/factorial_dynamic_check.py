def factorial(x: int) -> int:
    match x:
        case 0:
            return 1
        case n:
            ans = 1
            for i in range(1, n, 1):
                ans = ans * i
            return ans
