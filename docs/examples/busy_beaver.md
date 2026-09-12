⬅ [🏠 Home](../README.md)

⬅ [5 🧮 Worked Examples](README.md)

# 5.3 A Busy Beaver

The `n`-state busy beaver is the halting Turing machine of that size which runs longest.
The best for `n = 2` halts after 6 steps, having written 4 ones.

Transition table (`state, symbol: write, move, next`):

```
A, 0 -> 1, R, B
A, 1 -> 1, L, B
B, 0 -> 1, L, A
B, 1 -> 1, R, HALT
```

## Python

```python
def simulate_bb2(max_steps):
    tape = {}
    head = 0
    state = "A"
    steps = 0
    transitions = {
        ("A", 0): (1, 1, "B"),
        ("A", 1): (1, -1, "B"),
        ("B", 0): (1, -1, "A"),
        ("B", 1): (1, 1, "HALT"),
    }
    for _ in range(max_steps):
        if state == "HALT":
            break
        symbol = tape.get(head, 0)
        write, move, next_state = transitions[(state, symbol)]
        tape[head] = write
        head += move
        state = next_state
        steps += 1
    return steps, sum(tape.values())

print(simulate_bb2(20))
```

## Mamba

```mamba
# list/map index assignment isn't implemented yet
def simulate_bb2(max_steps: Int) -> Int := do
    def tape := {}
    def head := 0
    def state := 0
    def steps := 0
    for i in 0 .. max_steps do
        if state != 2 then do
            def symbol := if head in tape then tape(head) else 0
            def move := if state = 0 then (if symbol = 0 then 1 else -1) else (if symbol = 0 then -1 else 1)
            def next_state := if state = 0 then 1 else (if symbol = 0 then 0 else 2)
            tape(head) := 1
            head := head + move
            state := next_state
            steps := steps + 1
        end
    end
    return steps
end

print(simulate_bb2(20))
```

Bounded by `max_steps`, this is a `for` loop, and already fits `total`'s simplest rule:

```mamba
# total isn't implemented yet
def total pure simulate_bb2(max_steps: Int) -> Int := do
    ...
end
```

Whether an arbitrary machine halt at all is the halting problem.
`BB(n)` itself, the value as a function of `n`, is stronger still:
Radó proved in 1962 that no algorithm can compute it in general, since doing so would solve the halting problem. 
