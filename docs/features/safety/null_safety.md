⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.4 ⛑ Safety](README.md)

# 2.4.3 Null Safety

_Note_ `?` on a type, `None`, and `?` as a default operator are implemented.
The safe-call operator `?.` is future work, and is marked as such below.

We wish to be explicit about a function which may return nothing, as the user of the function might expect a value.
For this we use the question mark symbol: `?`

Take the following:

    # type error! 'lookup' might return nothing
    def my_function(names: Set[Str], name: Str) -> Str := lookup(names, name)

The type checker is complaining that `lookup` might return `None`.
To circumvent this, we make the return type of the function nullable.

    def my_function(names: Set[Str], name: Str) -> Str? := lookup(names, name)

Now when calling my function, I will either get a `Str` or a `None`.
Because this is explicit, we know this at compile time.

If we try to call a function or access a definition of the result directly we get a type error:

    # type error! called `is_digit` on an object which might be None
    def digit := my_function(names, "hello").is_digit()

_Note_ Calling a method only if the value is not `None`, with the safe-call operator `?.`, is future work.

## Default values

In some situations, we want to have a default value.
In such situations, we use the `?` operator.
Note that both sides of the operator must be of the same type.

    def maybe_world: Str? := my_function(names, "world")
    def world := maybe_world ? "world"

    # here, world is of type Str

    def other := my_function(names, "other")

    # here, other is of type Str?, as we do not know whether it is a Str or None

You can also return `None` in a function:

    def special_function(x: Int) -> Int? := if x > 10 then x else None
