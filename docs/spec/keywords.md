⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.2 Keywords

The following is a list of all the keywords in the language.

## Imports

Keyword | Use
---|---
`from`  | Specify where to import from
`import`| Specify what to import
`as`    | Specify alias of import

## Classes

Keyword | Use
---|---
`type`  | Denote start of type defintion or type alias
`when`  | Conditionals of a type definition
`class` | Denote start of class definition
`trait` | Denote start of trait definition

## Classes and Utils

Keyword | Use
---|---
`self`    | Refer to definitions of this class
`init`    | The constructor of the class
`forward` | Forwarding methods of contained class

## Definitions and Functions

Keyword | Use
---|---
`def`     | Denote definition
`fin`     | Denote defined variable is immutable
`pure`    | Denote function is pure
`total`   | Denote a function is total
`meta`    | Denote a meta function (evaluated during compile time)

## Boolean operators

Keyword | Use
---|---
`not`   | Negation of a boolean value
`and`   | And operator
`or`    | Or operator

## Mathematical Operators

Keyword | Use
---|---
`mod`   | Modulus operator
`sqrt`  | Square root operator

## Control flow Expressions

Keyword | Use
---|---
`if`      | Denote start of if expression or statement
`then`    | Denote start of then branch of if
`else`    | Denote start of else branch of if
`match`   | Denote start of a match expression or statement
`recover` | Recover from error, for (partial) local error recovery

## Control Flow Statements

Keyword | Use
---|---
`while`   | Denote start of while statement
`for`     | Denote start of for statement
`in`      | Specify which collection to iterate over in for statement
`do`      | Specify what needs to be done in control flow statement
`continue`| Continue onto next iteration within loop
`break`   | Exit loop

## Statements

Keyword | Use
---|---
`return`  | Return from a function or method
`pass`    | Empty placeholder statement
