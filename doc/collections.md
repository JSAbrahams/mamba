# Collections

The grammar has Four types of collections:
* Sets
* Lists
* Tuples
* Maps

A collection may be mutable. This however does not mean that its items are mutable. To make the items mutable as well 
the `ofmut` keyword must be used.

### Set
A set is created using either `{` and `}`, or the set-builder.

A set is an unordered collection of unique items.

    # A set may be immutable
    def animals <- { "dog", "cat", "mouse" }
    # Or mutable
    def mut instruments <- { "piano", "violin", "flute" }    
    
    # You can also define an immutable set of mutable variables
    def countries ofmut <- { "Netherlands", "Belgium", "Luxembourg" }
    # Or you can even define a mutable set of mutable variables
    def mut composers ofmut <- { "Beethoven", "chopin", "mozart" }
  
    # I can iterate over a set
    for animal in animals print animal
    
    # I can add an item to a mutable set
    instruments add "trombone"
    # Or remove an item from a mutable set
    instruments remove "violin"
    
    # I can change the values of items in a set of mutable items
    countries get "Netherlands" add " - Amsterdam" 
    for country in countries println country
    # The above prints:
    #  Netherlands - Amsterdam
    #  Belgium
    #  Luxembourg
    
### List
A list is created using `[` and `]`, or a list-builder.

A list can be accessed using an index: `list(1)`\
A list may be ordered.

### Tuple

A tuple is created using `(` and `)`.

A tuple a collection of named items.

### Map

A map is created using `{` and `}`, where each mapping is represented as such: `key -> value`, or a map-builder.

A map is a collection of unordered set of items.
