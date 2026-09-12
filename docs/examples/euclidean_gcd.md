⬅ [🏠 Home](../README.md)

⬅ [5 🧮 Worked Examples](README.md)

# 5.1 Euclidean Algorithm (GCD)

The oldest algorithm still in common use: the greatest common divisor of `a` and `b` is the greatest common
divisor of `b` and `a mod b`, down to a base case of `b = 0`.

## Python

```python
def gcd(a: int, b: int) -> int:
    if b == 0:
        return a
    return gcd(b, a % b)
```

Python gives no static guarantee this halts.
You'd know it does by the same argument a mathematician would give on paper: 
`a mod b` is always strictly smaller than `b`, and it can't go below `0`, so the second argument can only shrink finitely many times before hitting the base case.

## Mamba

```mamba
def gcd(a: Int, b: Int) -> Int := 
    if b = 0 then a
    else gcd(b, a mod b)

print(gcd(48, 18))
print(gcd(1071, 462))
```

Typed with `PosInt` (non-negative `Int`) and annotated fully, this reads:

```mamba
# PosInt and total aren't implemented yet
def total pure gcd(a: PosInt, b: PosInt) -> PosInt := 
    if b = 0 then a
    else gcd(b, a mod b)
```
