⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.1 🔀 Control Flow](README.md)

# 2.1.1 Control Flow Expressions

## If Expressions (or Statements)

If expressions are perhaps one of the most well known and universal programming constructs.

An if has either the form:

    if <expression> then <expression or statement>

or:

    if <expression> then <expression or statement> else <expression or statement>

For instance:

    if b = 3 then print("b is three.") else print("b is not three.")

A branch may also be a block, which is written between `do` and `end`:

    if b = 3 then do
        print("b is three.")
        print("which is a good number.")
    end else
        print("b is not three.")

An `if` is an expression if:

- Has an `else` branch.
- Return an `<expression>` (not a `<statement>`) in both branches.
  An expression evaluates to a value, for instance, `10 * x` is an expression if `x` is a number for instance, whereas `print("hello world")` is a statement, as it does not return anything.

So, an `if` _expression_ has the form

    if <expression> then <expression> else <expression>

An example would be:

    let my_value := if a > 0 then 2E30 else 8E21

### Match Expressions (or Statements)

A `match` can be used to match based on the value of an expression.

A `match` has the form:

    match <expression> where <one-or-more-cases> end

Where each case has the form `<expression> [ if <guard> ] => <expression or statement>`.
An example would be (if `b` is a number):

    match b where
        1 => print("one")
        4 => print("four")
        5 => print("five")
        _ => print("anything else")
    end

The last arm is the default case.
`_` matches one thing and binds nothing.

#### Every `match` must be exhaustive

Every possible value of the matched expression must be covered by some arm.
This holds whether the `match` is used as an expression or as a statement.
This is also to avoid confusing situations where users have to reason about whether a match is an expression or statement.

There are two ways to cover every value.
The first is a default arm, which is an arm whose pattern always matches.
That is `_`, a bare name, or a tuple whose elements all always match.
The second is to write out every value of a type that has finitely many:

    def label(b: Bool) -> Str := match b where
        True  => "yes"
        False => "no"
    end

`Bool` has exactly two values, and both are covered, so no default arm is needed.
A nullable type adds `None` to its values, and a tuple has every combination of its elements' values.
A guarded arm never counts towards covering a value, since its guard can fail.

A `match` that misses a value is rejected, with an example of one it misses:

    def both(p: (Bool, Bool)) -> Int := match p where
        (True, _)     => 1
        (False, True) => 2
    end

Here `(False, False)` is not covered.

#### Every arm must be reachable

An arm whose values are all covered by the unguarded arms above it can never match, and is rejected.
That includes any arm after a default arm, and a repeated arm.

#### Tuple patterns

A tuple pattern matches element by element.
A literal compares, and a name binds to the element in its place, with that element's type:

    def quadrant(point: (Int, Int)) -> Str := match point where
        (0, 0) => "origin"
        (x, y) => "at height {y}"
    end

A pattern name may shadow an outer one, and then refers to the element within its arm.
After the `match`, the outer name holds its own value again.

#### Guards

An arm may carry a guard, which is an extra condition checked after the pattern matches:

    def sign(x: Int) -> Str := match x where
        n if n < 0 => "negative"
        n if n > 0 => "positive"
        _          => "zero"
    end

The guard is checked after the pattern, so it can read what the pattern bound.
Here `n` is bound by the pattern and then tested by the guard.
A guard is an expression and must evaluate to a `Bool`.

A `match` is an expression if every arm returns an `<expression>` rather than a `<statement>`.
An expression evaluates to a value, whereas a statement does not.
