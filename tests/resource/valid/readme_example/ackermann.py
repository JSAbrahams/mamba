def ackermann(m: int, n: int) -> int:
    match (m, n):
        case (m, n) if m == 0:
            return n + 1
        case (m, n) if n == 0:
            return ackermann(m - 1, 1)
        case (m, n):
            return ackermann(m - 1, ackermann(m, n - 1))


print(ackermann(2, 3))
