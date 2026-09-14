⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.2 📝 Data](README.md)

# 2.2.3 Operator Overloading

We can overload the following operators of the language:

- `sqrt`
- `+` and `-`
- `*`, `/`, and `//`
- `^` and `mod`
- `=`
- `<` and `>`

The remaining comparison operators, `!=`, `<=`, and `>=`, cannot be given a body of their own.

Overloading operators gives us the ability to more concisely work with more complex objects.
To demonstrate the concept of operator overloading we will use the (incomplete) `Vec2` class, which represents a two-dimensional vector.
Note that we deliberately do not use `Complex` here: that is a built-in primitive type, so a class of that name would clash with it.

Say we define a `Vec2` class as such:

```mamba
class Vec2(x: Float, y: Float) where
    def +(self, other: Vec2) -> Vec2 := Vec2(self.x + other.x, self.y + other.y)

    # we can also overload a unary operator
    # when overloading, the default return value is the type itself, in this case Vec2
    def sqrt(self) -> Vec2 := Vec2(sqrt self.x, sqrt self.y)

    def to_string(self) -> Str := "({self.x}, {self.y})"
end
```

Note that, as with any method, an overload takes an explicit `self` argument.

Now we can use the `Vec2` as follows:

```mamba
from vec2 import Vec2

def a := Vec2(1.0, 2.0)
def b := Vec2(2.0, 3.0)

# the `+` operator of Vec2 has been overloaded
def c := a + b
print(c.to_string()) # prints (3.0, 5.0)
```

_Note_ The above type checks, and the overloads generate as Python's `__add__` and friends.
Generating a *runnable* module from it still hits two known generator bugs, though: annotating a method with its own class type (`other: Vec2`) emits a forward reference Python rejects, and without `--annotate` a single-expression body loses its `return`.
See [tests/README.md](../../../tests/README.md) for both.
