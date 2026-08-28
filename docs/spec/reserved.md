⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.2 Special Characters and Symbols

## 3.2.1 Keywords

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
`type`  | Denote start of a type refinement / conditional type alias, or interface-signature body (experimental)
`when`  | Conditionals of a type definition (`type` only, not `trait`)
`class` | Denote start of class definition
`trait` | Denote start of trait (interface) definition

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

## Control Flow Statements

Keyword | Use
---|---
`while`   | Denote start of while statement
`for`     | Denote start of for statement
`in`      | Specify which collection to iterate over in for statement
`do`      | Specify what needs to be done in control flow statement, and start of code block
`continue`| Continue onto next iteration within loop
`break`   | Exit loop

## Statements

Keyword | Use
---|---
`return`  | Return from a function or method

## Blocks

Keyword | Use
---|---
`end`   | Denote end of code block or set
`where` | Denote start of code set
`using` | Denote start of a resource block, binding an alias for the duration of its body

## 3.2.2 Special Characters

The following is a list of characters in the language

## Brackets

Symbol | Use
---|---
`(` | Denote start of tuple elements or function arguments, including collection indexing (list or mapping)
`)` | Denote end of tuple elements or function arguments, including collection indexing (list or mapping)
`{` | Denote start of set, set constructor, or map
`}` | Denote end of set, set constructor, or map
`[` | Denote start of list, or opening bracket of generics of a class
    | Also denotes the start of a list of statements and/or expressions
`]` | Denote end of list, or closing bracket of generics of a class
    | Also denotes the end of a list of statements and/or expressions

## Type

Symbol | Use
---|---
`?` | Denote optional type

## Mathematical Operators

Symbol | Use
---|---
`*` | Multiply operator
`^` | Power operator
`-` | Minus operator
`+` | Plus operator
`/` | Division operator

## Boolean Operators

Symbol | Use
---|---
`>`  | Greater than operator
`>=` | Greater than or equal to operator
`<`  | Less than operator
`<=` | Less than or equal to operator
`=`  | Structurally equal
`!=` | Structurally not equal

## Assignment and Functions

Symbol | Use
---|---
`:=` | Assign to definition
`->` | Used in signature of method or function
`=>` | Denote the mapping in a match arm
`:`  | Specify type of identifier
`.`  | Precedes method call, or fractional digits of real number
`,`  | Separates arguments in collections or function or method signatures
`_`  | Anonymous value

## Reassignment Operations

Symbol | Use
---|---
`+=`  | Add value to variable and assign to variable
`-=`  | Subtract value from variable and assign to variable
`*=`  | Multiply value with variable and assign to variable
`/=`  | Divide variable by value and assign to variable
`^=`  | Raise variable by value and assign to variable

## Context Dependent

Symbol | Use
---|---
`E` | If nested between two integers, or an integer and a real, denotes e-number
'\|' | Within set and list builder notation "such that"

## Other Operators

Symbol | Use
---|---
`..`  | Exclusive range, or range step
`..=` | Inclusive range
`::`  | Exclusive slice, or slice step
`::=` | Inclusive slice

## Comments

Symbol | Use
---|---
`#`  | Start of a comment
`##` | Start and end of comment block
