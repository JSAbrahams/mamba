# Util

A util is a collection of immutable definitions, which is either an immutable value, or a function. As such, a util is a
stateless collection of functions with no side-effects. Any definition may be private. A util may not be instantiated,
only used.

I could for instance define a Util with some helper math functions:

    TrigRange isa Real where
        self <= 1 else Err("Not inside valid range.")
        self >= -1 else Err("Not inside valid range.")

    util Math
        def cos (num: Real): TrigRange ->
        def cos (num: Radians): TrigRange ->