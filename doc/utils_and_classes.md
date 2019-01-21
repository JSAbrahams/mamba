# Utils and Classes

### Module

A module is one of the following:

* Be a `type`
* Be a `util`
* Be a `util` and a `class`
* Be a `script`

A file may contain multiple modules.

### Type

Describes the properties a class should have that implements this type. A type contains a collection of method
signatures and immutable variables.

Everything in a type is publicly visible.

### Util

A util is a collection of immutable definitions. A definition here is either an immutable value, or a function. A 
definition may be private, unless it is a property of the type itself.

A util may not be instantiated, only used.

### Class

A class describes the properties of an instance of that class. A class encapsulates data and contains definitions. A 
definition is either a value, or an immutable method. A definition may be private.

A methods differs from a function in that it can modify the fields of an instance of the class.

An instance of a class may be created. A class may not inherit from another class, but may have another class as a 
property, and forward its methods.

A class may implement a type. It must then either implement the methods described in the type, or contain an instance of
a class that forwards these methods.

A class `MyClass`, with no arguments, is instantiated as such:\
`def my_class = MyClass()` or `def mut my_class = MyClass()`

A immutable instantiation may not modify its fields. Therefore, it is not possible to call a method of an instantiation 
that modifies its fields.

A method modifies the fields of an instance by assigning to a variable preceded by `self`. All fields of a `class` must 
be either defined in the constructor or at the top of the body of the `class`.

`to_string` and `to_hash` of a class may be either a function or a constant. If these are not implemented, the default
implementation is used.

`eq` may be implemented. If not, the default implementation is used of recursively checking that the values are equal.
For contained instances of the classes, this is only done for forwarded definitions. See "Operator Overloading" for more
details on operator overloading.

### Script

A script is the runnable of an application, this is what is executed.

A script may contain function definitions. A script may contain a single code block.
