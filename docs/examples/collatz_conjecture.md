⬅ [🏠 Home](../README.md)

⬅ [5 🧮 Worked Examples](README.md)

# 5.2 The Collatz Conjecture

Take any positive integer.
If it's even, halve it; if it's odd, triple it and add one.
Whether this always reaches `1` is an open conjecture.

## Python

```python
def collatz_steps(n: int) -> int:
    steps = 0
    while n != 1:
        if n % 2 == 0:
            n = n // 2
        else:
            n = 3 * n + 1
        steps += 1
    return steps
```

## Mamba

```mamba
def collatz_steps(n: Int) -> Int := do
    if n = 1 then return 0

    if n mod 2 = 0 
        then 1 + collatz_steps(n // 2)
        else 1 + collatz_steps(3 * n + 1)
end

print(collatz_steps(27))
```

This can never be marked `total`: the `3 * n + 1` branch grows `n`, and no decreasing measure is known, since none is known to exist.
Proving termination here is the Collatz conjecture itself.
