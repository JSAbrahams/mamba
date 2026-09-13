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
`trait` | Denote an interface (à la Java interfaces / Rust traits), which a `class` can implement
`class` | Denote a class
`when`  | Conditional types (used with `type`, not `trait`)

## Classes and Utils

Keyword | Use
---|---
`self`    | Refer to definitions of this class
`forward` | Forwarding methods of contained class

## Definitions and Functions

Keyword | Use
---|---
`def`     | Denote definition
`mut`     | Denote binding is mutable
`pure`    | Denote function is pure

## Boolean operators

Keyword | Use
---|---
`not`   | Negation of a boolean value
`and`   | And operator
`or`    | Or operator

## Mathematical Operators

Keyword | Use
---|---
`mod`   | Modulus operator, which resolves to `__mod__` on the left operand

## Membership

Keyword | Use
---|---
`in`    | Membership test, as in `1 in [1, 2]`

## Control flow Expressions

Keyword | Use
---|---
`if`    | Denote start of if expression or statement
`then`  | Denote start of then branch of if
`else`  | Denote start of else branch of if
`match` | Denote start of a match expression or statement

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

## Blocks

Keyword | Use
---|---
`end`   | Denote end of code block or set
`where` | Denote start of code set
`using` | Denote start of a resource block, binding an alias for the duration of its body
