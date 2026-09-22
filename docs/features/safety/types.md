⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.4 ⛑ Safety](README.md)

# 2.4.1 Types

_Note_ Static typing and type inference, covered first, are implemented.
Type aliases and type refinement, covered from "Type Aliases and Type Refinement" onwards, are future work.
That includes the `type ... when ...` notation, the `is_instance` function, and `as` casts.

Often a distinction is made between static and dynamic typing.

If the application were dynamically typed, we would constantly have to verify that variables are indeed what they claim to be.
Say I have a variable `beethoven`, can I assume that it is an instance of `Composer`?

    # how can I be sure that this function argument is a composer?
    def my_function(composer) := composer.composer_method()

There are of course ways to check this, such as using the `isinstance` method in Python, but this is rather tiresome.
To this end, we use types.
A user defines a class `Composer`, which defines the behaviour of a composer:

    class Composer(name: Str) where
        def composer_method(self) -> Int := 10
    end

And then we define the `my_function` as such:

    def my_function(composer: Composer) -> Int := composer.composer_method()

Now, in the body of the function, we can rest easy knowing that the passed variable is indeed a composer.
It is actually now impossible to pass another variable type to the function, as this is statically checked by the type checker.
If it sees that we try to pass something that is not a composer, it will give an error, meaning that the program will not run.

In some programming languages, we have to explicitly state the type of each variable.
This however makes the application rather verbose.
Take for instance:

    def x: Int := 10                     # x is obviously an integer
    def c: Complex := Complex(10, 20)    # from the right hand side it is already clear that c is complex

Instead, we can use type inference.
The type of every variable is inferred from the context in which it is used.

    def x := 10                 # x has type Int, we know this because 10 is an Int
    def c := Complex.new(10, 20)    # c has type Complex
    def y := 20.1               # 20.1 uses decimal notation, so we know y is a Float

    def z: Float := 10.5        # In some situations however, you still might want to explicitly mention the type

The program is still statically typed, but now we don't require the developer to write everything out in full.

## Unbounded integers

`Int` is arbitrary-precision by design.
It has no width, no maximum, and no wrapping behaviour.
A number that silently wraps is a correctness bug.
`Nat`, the non-negative integers, is a refinement of `Int` and inherits this.
`Nat` is future work, see [`Nat`](#nat) below.

Compiling to machine code with `--bin` or `--asm` lowers `Int` to a fixed-width 64-bit integer.
However, the two backends are meant to agree.
The divergence is therefore a known limitation, not intended behaviour.
Arbitrary-precision arithmetic in the Cranelift backend is future work.

## Type Aliases and Type Refinement

_Note_ Everything from here on is future work.

We can also use type aliases and type refinement to further refine types by adding conditions to them.
Say we have the following:

    type DeadComposer: Composer when
        self.death_date != None else "Composer is not dead."

We now rewrite my_function so it only works for `DeadComposer`s:

    def my_function(composer: DeadComposer) -> Int := today.year - composer.death.year

Again, we can rest assured that `composer` is a `DeadComposer` in the body of the function.
To use such a function, we must explicitly cast a `Composer`:

    def chopin := Composer("Chopin")

    if is_instance(chopin, DeadComposer) then do
        def years_ago := my_function(chopin)                    # chopin is dynamically casted to a DeadComposer
        print("{chopin.name} died {years_ago} years ago.")
    end

This draws on concepts of **Design by Contract** philosophy.

Furthermore, it also allows us to explicitly define the state of an object, something which is often left ambiguous.
For instance, we can say a matrix is invertible or singular by doing the following:

    trait Matrix where
        def determinant(self) -> Float
        def solve(self, u: Float, v: Float) -> List[Float]
    end

    type InvertibleMatrix: Matrix when
        self.determinant() != 0.0 else "Matrix is singular"

And we may then elsewhere implement this `Matrix` trait:

    class Matrix2x2(a: Float, b: Float, c: Float, d: Float): Matrix where
        def determinant(self) -> Float := self.a * self.d - self.b * self.c

        # You can only call this function if I am an invertible matrix
        def solve(self: InvertibleMatrix, u: Float, v: Float) -> List[Float] := pass
    end

This is a rather trivial example, but it shows how we can explicitly name the different states of a matrix.

## Type aliases

In some cases, for readability we might want to write a type alias.
Say we have the following method:

    def distance_remaining(self, covered: Int) -> Int := self.total - covered

The above seems simple, but there are two issues:

* At a glance, we cannot know what covered symbolises.
  Kilometers, meters?
  We can of course rename the variable, but in certain situations this makes the code rather verbose.
* We do no bounds checking here.
  What if covered is more than the total, or negative?
  We could add these bounds checks to the method.
  However, this makes the method more verbose.
  Ideally, we want the method to express in a concise manner what it does without having a majority of the method being error handling code.

To solve the above two issues, we can use type aliases.
Observe the following:

    type Kilometer: Int

Type `Kilometer` can do everything an `Int` can (we can use all the same operators), but using such an alias allows us to more clearly express our ideas in the codebase without relying on documentation.
(This is a recurring theme, source code ideally should speak for itself without relying heavily on documentation.)
We can rewrite the method as so:

    def distance_remaining(self, covered: Kilometer) -> Kilometer := self.total - covered

## Type Refinement

Type refinement expands upon type aliases by defining certain conditions an object must adhere to, to be considered that type.
This can also be used to enforce pre-conditions of a function or method when used as a parameter, and post-conditions when used as the return type.
This is akin to the philosophy of Design by Contract.

Say we have a function:

    def f(x: Int) -> Int := do
        print("this number is even: {x}")
        x
    end

In some situations, this function does not behave as we expect it to.
It may print an uneven number.
In such a situation, we often turn to the design by contract philosophy, where a function has pre and post-conditions.
There are several traditional approaches to solving this problem, both of which are valid, though the preferred approach does depend on context:

Just return `x` if it is uneven and don't print anything using a simple `if`:

    def f(x: Int) -> Int := do
        if x mod 2 != 0 then return x
        print("this number is even: {x}")
        x
    end

Raise an error if `x` is uneven:

    def f(x: Int) -> Int ! Err := do
        if x mod 2 != 0 then ! Err("Expected x to be even.")
        print("this number is even: {x}")
        x
    end

However, in the above code, we see that writing pre-conditions can get out of hand, and we might want to use the same pre-conditions for multiple functions, which results in duplicate code.
This is a situation where type aliases with conditions can come in handy.
Type aliases encourage decentralisation.
The logic of a type is closely linked to the type itself, instead of having to manually check that a type adheres to certain conditions every time it is used.

We can use a trivial type `EvenNum` to demonstrate how one would use conditions in a type alias.
Say we define the type-alias `EvenNum`:

    type EvenNum: Int when
        self mod 2 = 0 # we can list more conditions below this one. They must all evaluate to a boolean.

We may also choose to add a descriptive error message:

    type EvenNum: Int when
        self mod 2 = 0 else "{self} is an uneven number"

This defines all `Int`, or Integers, that are even.
That is, the condition listed above holds.
This is similar to creating a new class `EvenNum` which is an `Int`, and verifying that these properties hold.

We can now redefine the function as follows:

    # EvenNum has conditions, so we need to state that it may raise an error
    def g(x: EvenNum) -> Int ! Err := do
        print("this number is even: {x}")
        x
    end

Now, the actual type of the argument describes what conditions the argument adheres to, instead of having to manually check these in the body of the function.
We now know that these conditions hold in the body of the function.
We can cast any variable that is an `Int` to `EvenNum`.
During casting, the defined conditions are checked, and the respective error is thrown if a condition does not hold:

    # We can cast x to an EvenNum, which might give an error
    def x := random_int() # here x is an Int
    def y := x as EvenNum
    def first := g(y)

    # We can also pass it immediately if we want, in which case it is casted to EvenNum
    def z := random_int()
    def second := g(z)
    # which is the same as
    def second_with_cast := g(z as EvenNum)

    # Or just say that the variable is an EvenNum upon instantiation
    def y: EvenNum := random_int()
    def third := g(y)

    # We can also use is_instance to check that the conditions hold without raising an error
    def a := random_int()
    # notice how we don't have to cast a to an EvenNum if the condition holds.
    # We know that the then branch of the if is only executed if a is an EvenNum, so we assign it the type EvenNum
    def fourth := if is_instance(a, EvenNum) then g(a) else 0

    # If it can be statically verified that the properties hold, it is not necessary to handle any type specific errors
    def c := 2
    def fifth := g(c)

    # first, second, third, fourth, and fifth all have type Int

Note that `a` is an `Int`, and nothing casts it, yet `g(a)` is accepted.
That branch only runs when `is_instance(a, EvenNum)` held, so `a` is an `EvenNum` there and an `Int` everywhere else.
Narrowing a type from what was declared to what a particular branch guarantees is called flow narrowing.
See [Flow narrowing](#flow-narrowing) for more details.

We can also use it as a sort of post-condition of the function.
We ensure that the function returns an `EvenNum`:

    def g(x: EvenNum) -> EvenNum ! Err := do
        print("this number is even: {x}")
        def y := x + some_other_function(x)
        y as EvenNum
    end

Or:

    def g(x: EvenNum) -> EvenNum ! Err := do
        print("this number is even: {x}")
        def y: EvenNum := x + some_other_function(x)
        y
    end

We can even ensure that the function never returns an error:

    def h(x: EvenNum) -> EvenNum := do
        print("this number is even: {x}")
        def y := x + some_other_function(x)
        if is_instance(y, EvenNum) then
            y # flow narrowing ensures that this is an EvenNum
        else x
    end

So now:

    def x := 10  # here x is an Int
    def a := g(x as EvenNum)

    def b := g(a) # we don't have to cast a to an EvenNum, it is already of that type

    def c := h(x)  # function h never raises an error

### `Nat`

`Nat`, the non-negative integers, is defined as a refinement rather than as a separate primitive:

    type Nat: Int when
        self >= 0 else "{self} is negative"

Zero is in the set.
That matters wherever `Nat` is used as a measure, since `0.abs()` and `"".len()` are both `0`.

`Nat` is closed under addition but not under subtraction.
Subtracting a larger `Nat` leaves the domain, so an operation that may do so returns `Nat?`.
See [Total Functions](../functions/total_functions.md#partial-subtraction-in-strictlydecreases) for the case that motivates it.

### Flow narrowing
Flow narrowing is what the checker knows at one particular point, given the tests that guard it.
A test that a value satisfies a refinement narrows that value in the branch where the test holds.
In the branch where it fails, it narrows to the complement, where that complement can be written down.

This is also why `is_instance` is a function the compiler knows about rather than an ordinary call.
An ordinary call returns a `Bool` and tells the checker nothing about its argument.

So far every example has narrowed on an `if`.
Facts reach a point from three places.
The first is the branches of an `if`.
The second is the guard on a `match` arm, which holds for that arm's body.
The third is the negated guards of every earlier arm, which hold for every later one, because a later arm is only reached when all the earlier ones failed.

That third source is the one that does the most work:

    def classify(x: Str?) -> Str := match x where
        n if n = None => "nothing"
        n             => n.upper()   # n is Str here, since the arm above did not match
    end

Flow narrowing is deliberately limited.
It narrows membership of a union or of a refinement.
It does not reason about arithmetic.
Knowing `m != 0` tells it nothing about `m - 1`, because that is a fact about a derived value rather than about `m` itself.
Closing that gap is what the next section is for.

All of this is erased at compile time.
Narrowing changes what the checker accepts, never what the generated Python does.

### Interval refinement

`Nat` is `Int` refined by `self >= 0`.
Flow narrowing alone cannot type Ackermann's function, which is the case [Total Functions](../functions/total_functions.md) is built around:

    def ackermann(m: Nat, n: Nat) -> Nat := match (m, n) where
        (m, n) if m = 0 => n + 1
        (m, n) if n = 0 => ackermann(m - 1, 1)
        (m, n)          => ackermann(m - 1, ackermann(m, n - 1))
    end

Read flow-insensitively, `m - 1` has type `Nat?`, since subtraction can leave the domain.
That forces the whole signature down to `Int`.
Read with the guards in hand, it cannot: the second arm is only reached when `m = 0` failed.

Interval refinement is the smallest addition that closes this.
Every integer carries a known range at each point in the program, and `Nat` is the range `Int[0..]`.
Ranges narrow on comparisons, and join back together where branches merge.

For the three arms above:

| arm | known on entry | obligation | discharged by |
|---|---|---|---|
| `if m = 0` | `m, n` in `[0..]` | `n + 1` is `Nat` | `[0..] + 1` is `[1..]`, inside `[0..]` |
| `if n = 0` | `m` in `[1..]` | `m - 1` is `Nat` | `[1..] - 1` is `[0..]` |
| catch-all | `m, n` in `[1..]` | `m - 1`, `n - 1` are `Nat` | same, for each |

One detail decides whether this works at all.
Negating `m = 0` gives a disequality, and a range cannot represent "everything except zero".
It narrows here only because `Int` is discrete, so `m >= 0` together with `m != 0` gives `m >= 1`.
A disequality against a range endpoint has to be handled for that step, or Ackermann silently fails to narrow and the feature does nothing for the one example that motivates it.

Ranges are not relational.
A range records what is known about one value against constants.
It cannot record how two values relate, so this does not typecheck even though it holds:

    def difference(a: Nat, b: Nat) -> Nat := if a >= b then a - b else 0

`a >= b` is a fact about a pair.
Recovering it needs either a relational domain, which tracks bounds on `a - b` directly, or a solver.
That is a larger step, and it is the one that also covers `StrictlyDecreases.subtract`, whose guard has exactly this shape.

Whichever is chosen, the arithmetic stays restricted to `Add`, `Sub`, `Eq` and `Comparable`.
That is the same Presburger fragment [Total Functions](../functions/total_functions.md#measurable-and-custom-types) already requires of `Measurable`, and for the same reason.
Multiplication between two non-constant values leaves it and is the hard boundary in both cases.

This stays a narrowing rather than an inference, in keeping with the rest of the language.
A signature is still written out in full.
The checker either discharges the obligation or reports the one it could not, naming the facts it had:

    Cannot prove 'm - 1' is Nat
      m is in [1..] here, from Nat and from 'm = 0' failing on the arm above

Both sections above are future work.

**Further reading:**

- P. Cousot, R. Cousot, *Abstract Interpretation: A Unified Lattice Model for Static Analysis of Programs by Construction or Approximation of Fixpoints*, POPL 1977.
- A. Miné, *The Octagon Abstract Domain*, Higher-Order and Symbolic Computation 19, 2006.
- P. Rondon, M. Kawaguchi, R. Jhala, *Liquid Types*, PLDI 2008.
- N. Vazou, E. Seidel, R. Jhala, D. Vytiniotis, S. Peyton Jones, *Refinement Types for Haskell*, ICFP 2014.
- S. Tobin-Hochstadt, M. Felleisen, *Logical Types for Untyped Languages*, ICFP 2010.
