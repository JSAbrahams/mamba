⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.2 📝 Data](README.md)

# 2.2.1 Collections

We have three types of collections:
- `Set`
- `List`
- `Tuple`

We can also create key-value pairs as such:
`<expression> => <expression or statement>`
If Mamba detects we are building a data-structure which only such values, it becomes a dictionary.

By default, a collection is immutable, meaning that we cannot add or remove items, or make changes to the contained
A collection may be mutable.

### Set

A set is created using either `{` and `}`, or by using the set-builder notation.
A set is an unordered collection of unique items, meaning that we cannot access an item using its index.

Below we show some examples of how a set may be used:
```
    # A set may be immutable
    def animals <- { "dog", "cat", "mouse" }
    # Or mutable
    def mut instruments <- { "piano", "violin", "flute" }      
    # I can iterate over a set
    for animal in animals print animal
    
    # I can add an item to a mutable set
    instruments add "trombone"
    # Or remove an item from a mutable set
    instruments remove "violin"
    # Or modify items in a mutable set
    foreach item in instruments do item += " is an instrument"
```

### List
A list is created using `[` ... `]`, or by using the list-builder notation.

A list can be accessed using an index: `list(1)`\
A list may be ordered.

### Tuple

A tuple is created using `(` ... `)`.
A tuple a collection of named items.

### Dictionary

A map is created using `{` ... `}`, where each mapping is represented as such: `key => value`.
A map is a collection of unordered set of items.
