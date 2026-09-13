⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.2 📝 Data](README.md)

# 2.2.1 Collections

We have three types of collections:

- `Set`
- `List`
- `Tuple`

We can also create key-value pairs as such:
`<expression> => <expression>`
If Mamba detects we are building a data-structure which only contains such values, it becomes a dictionary.

As with any other definition, a collection is mutable unless we mark it `fin`.
A `fin` collection cannot be reassigned.

## Set

A set is created using either `{` and `}`, or by using the set-builder notation.
A set is an unordered collection of unique items, meaning that we cannot access an item using its index.

Below we show some examples of how a set may be used:

    # A set may be immutable
    def fin animals := { "dog", "cat", "mouse" }
    # Or mutable
    def instruments := { "piano", "violin", "flute" }

    # I can iterate over a set
    for animal in animals do print(animal) end

    # I can check whether a set contains an item
    if "dog" in animals then print("there is a dog")

    # I can reassign a mutable set
    instruments := { "piano", "trombone" }

    # I can derive a new set from an existing one using set-builder notation
    def described := { item + " is an instrument" | item in instruments }

_Note_ The standard library does not define methods on a `Set` yet, so there is no `add` or `remove`.
Iteration, containment with `in`, and the set-builder notation above are what is available today.

## List

A list is created using `[` ... `]`, or by using the list-builder notation.

A list is ordered, and can be accessed using an index: `list(1)`.
Note that we index using round brackets, and not square ones.

    def xs := [ 4, 9, 16 ]
    print(xs(0)) # prints '4'

## Tuple

A tuple is created using `(` ... `)`.
A tuple is a fixed-size collection, where each item may have a different type.

    def t := (1, "two")

We can also define several variables at once by destructuring one:

    def (a, b) := (10, 20)

## Dictionary

A map is created using `{` ... `}`, where each mapping is represented as such: `key => value`.
A map is an unordered collection of items.
As with a list, we index it using round brackets.

    def pairs := { "do" => 1, "ree" => 2 }
    print(pairs("do")) # prints '1'
