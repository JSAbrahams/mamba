⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.2 Keywords

The following is a list of all the keywords in the language.

### Imports

Keyword | Use 
---|---
`from`  | Specify where to import from
`import`| Specify what to import
`as`    | Specify alias of import

### Classes

Keyword | Use 
---|---
`type`  | When constructing an interface (abstract base class) or type alias
`class` | Denote a class
`isa`   | Check whether an object is instance of a class

### Classes and Utils

Keyword | Use 
---|---
`self`    | Refer to definitions of this class
`init`    | The constructor of the class
`forward` | Forwarding methods of contained class

### Definitions and Functions

Keyword | Use 
---|---
`def`     | Denote definition
`private` | Denote private variable of class
`fin`     | Denote defined variable is immutable

### Boolean operators

Keyword | Use 
---|---
`not`   | Negation of a boolean value
`and`   | And operator 
`or`    | Or operator
`is`    | Check whether an instance is another instance
`isnt`  | Check whether an instance is not another instance
`True`  | True value
`False` | False value

### Mathematical Operators

Keyword | Use 
---|---
`mod`   | Modulus operator
`sqrt`  | Square root operator

### Control flow Expressions

Keyword | Use 
---|---
`if`    | Denote start of if expression or statement
`then`  | Denote start of then branch of if
`else`  | Denote start of else branch of if
`match` | Denote start of a match expression or statement

### Control Flow Statements

Keyword | Use 
---|---
`while`   | Denote start of while statement
`for`     | Denote start of for statement
`in`      | Specify which collection to iterate over in for statement
`do`      | Specify what needs to be done in control flow statement

### Statements

Keyword | Use 
---|---
`print`   | Print value of an expression
`println` | Print value of an expression
`return`  | Return from a function or method
`pass`    | Empty placeholder statement

### Errors

Keyword | Use 
---|---
`handle` | Denote handle cases
`raise`  | Denote that an expression, statement, or function may raise an error
`retry`  | Retry an expression from within handle case

### Special

Keyword | Use 
---|---
`None`  | Denote an undefined value
