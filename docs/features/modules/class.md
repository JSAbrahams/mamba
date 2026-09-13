⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.2 Class

## Fields are class arguments

A class declares its fields as arguments:

```mamba
class Point(x: Int, y: Int)
```

`x` and `y` are fields, stored on `self` and readable as `p.x` from outside.
There is no `def` prefix and no separate constructor to write.

This is the only way a value enters an object from outside.
Mamba has no `__init__`, and declaring one is an error.

## Construction goes through `new`

Every class gets a `new` taking exactly its class arguments:

```mamba
def p := Point.new(3, 4)
```

Applying the class to its arguments, as `Point(3, 4)`, is the underlying primitive.
It is in scope only within `Point` itself.
Outside it does not resolve, and the compiler says so, pointing you at `Point.new`.

That restriction is the whole point.
If the primitive were public, `new` would be a naming convention rather than a guarantee.
A caller could bypass any check it performs, and the same object would be built two different ways in the same codebase.

Because it is not public, declaring your own `new` gives you a constructor that cannot be sidestepped:

```mamba
class Fraction(num: Int, den: Int) where
    def new(num: Int, den: Int) -> Self ! FractionErr :=
        if den = 0 then ! FractionErr("Denominator is zero") else Fraction(num, den)
end
```

A declared `new` replaces the generated one.
Note that it may fail, which a constructor cannot.

## Associated functions

A function in a class body that takes no `self` is an associated function.
It is called on the class rather than on an instance.

`new` is one of these, which is what keeps it ordinary rather than special.
Named alternatives therefore sit beside it as equals:

```mamba
class Matrix2x2(a: Float, b: Float, c: Float, d: Float) where
    def identity() -> Self := return Matrix2x2(1.0, 0.0, 0.0, 1.0)
    def diagonal(a: Float, d: Float) -> Self := return Matrix2x2(a, 0.0, 0.0, d)
end

def m := Matrix2x2.identity()
```

`Self` names the enclosing class as a type.
It works in a method and in an associated function alike, in return position and in argument position.

## Derived fields

A field declared in the class body is derived.
It is computed from the class arguments rather than passed:

```mamba
class Circle(radius: Float) where
    def area: Float := self.radius * self.radius * 3.14159
end

def c := Circle.new(2.0)
```

This exists so that you need not name every field at the construction site.
`area` is computed once per instance and read as `c.area` like any other field.

A derived field must be assigned a value, unless its type is nullable.
Without one it would hold `None` whatever its type claims, which the type checker would then believe.
If you meant to pass the value rather than compute it, make it a class argument.

## The body holds declarations only

A class body may contain field declarations, method declarations, and a leading docstring.
A bare statement is rejected.

Such a statement would run once per instance, so it is really constructor code in disguise.
Leaving it there hides whether constructing the class has side effects.
Putting that work in an explicit `new` makes it visible in a signature instead.

This is also what lets construction be pure by default in the absence of an explicit `new`: there is nothing in a class body that can do anything.

## Pure construction

Purity is never inferred.
A class states that constructing it is pure by declaring `new` pure, with no argument list and no body:

```mamba
class Point(x: Int, y: Int) where
    def pure new
end

def pure origin() -> Point := return Point.new(0, 0)
```

There is no argument list because this is not a signature.
The generated `new` already has the class arguments, and this asserts a property of it.

The assertion is checked against the derived field initializers, since those are the only thing construction runs.
A field initialized by an impure call is rejected.

Leave the assertion out and `new` is an ordinary impure function, so a pure function may not construct the class.
Writing a bare `def new` asks for what the class already has, and the compiler warns that it is redundant.

### What `pure new` does not say

It constrains construction only.
Methods are untouched:

```mamba
class Counter(start: Int) where
    def pure new
    def mut count: Int := self.start

    def tick(mut self) := do
        self.count := self.count + 1
        print(self.count)
    end
end
```

`tick` mutates and prints, and that is fine, because no claim was made about it.

This is why the marker sits on `new` rather than on the class.
A class-level `class pure Counter` would read as though `tick` were pure too.
The narrower reading is the useful one, so it is spelled on the thing it actually constrains.

See [Pure Functions](../functions/pure_functions.md) for the rules a pure function obeys.
