# Keywords

The following is a list of all the keywords in the language.

### Imports
Keyword | Use 
--------|-----
`use`   | Specify functions when importing util
`useall`| Specify using all functions when importing util
`as`    | Specify function name when importing util

### Modules
Keyword | Use 
--------|-----
`type`  | When constructing an interface or type alias
`util`  | Denote a util
`class` | Denote a class
`isa`   | Check whether an object is instance of a class

### Classes and Utils
Keyword   | Use 
----------|-----
`self`    | Refer to definitions of this class
`init`    | The constructor of the class
`forward` | Forwarding methods of contained class

### Definitions and Functions
Keyword | Use 
--------|-----
`def`   | Denote definition
`mut`   | Denote that definition is mutable
`ofmut` | Denote that a collection contains mutable variables

### Boolean operators
Keyword | Use 
--------|-----
`not`   | Negation of a boolean value
`and`   | And operator 
`or`    | Or operator
`is`    | Check whether an instance is another instance
`isnt`  | Check whether an instance is not another instance
`eq`    | Check whether an expression evaluates to another expression
`neq`   | Check whether an expression does not evaluate to another expression
`true`  | True value
`false` | False value

### Mathematical Operators
Keyword | Use 
--------|-----
`mod`   | Modulus operator
`sqrt`  | Square root operator

### Other operators
Keyword | Use
--------|-----
`in`    | Check that an expression is contained within an expression which is a collection
`to`    | Denote a range
`toincl`| Denote an inclusive range

### Control flow Expressions
Keyword | Use 
--------|-----
`if`    | Denote start of if expression or statement
`then`  | Denote start of then branch of if
`else`  | Denote start of else branch of if
`when`  | Denote start of a when expression or statement

### Control Flow Statements
Keyword   | Use 
----------|-----
`while`   | Denote start of while statement
`foreach` | Denote start of for statement
`in`      | Specify which collection to iterate over in for statement
`do`      | Specify what needs to be done in control flow statement

### Statements
Keyword   | Use 
----------|-----
`print`   | Print value of an expression
`println` | Print value of an expression with a newline at the end
`return`  | Return from a function or method

### Errors
Keyword  | Use 
---------|-----
`handle` | Denote handle cases
`raise`  | Denote that an expression, statement, or function may raise an error
`retry`  | Retry an expression from within handle case

### Special
Keyword     | Use 
------------|-----
`undefined` | Denote an undefined value
