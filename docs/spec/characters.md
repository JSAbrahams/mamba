⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.3 Special Characters and Symbols

The following is a list of the symbols in the language.
These are exactly the symbols the lexer reads.

## Brackets

Symbol | Use
---|---
`(` | Denote start of tuple elements or function arguments, including collection indexing (list or mapping)
`)` | Denote end of tuple elements or function arguments, including collection indexing (list or mapping)
`{` | Denote start of set, set constructor, or map
`}` | Denote end of set, set constructor, or map
`[` | Denote start of list, or opening bracket of generics of a class
`]` | Denote end of list, or closing bracket of generics of a class

## Type

Symbol | Use
---|---
`?` | After a type, denote that it is optional; between two expressions, take the left unless it is `None`

## Mathematical Operators

Symbol | Use
---|---
`*` | Multiply operator
`^` | Power operator
`-` | Minus operator
`+` | Plus operator
`/` | Division operator
`//` | Floor division operator

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
`_`  | Anonymous value, and in `new(_)` one class argument that is not written out
`\`  | Start of an anonymous function, as in `\x: Int := x + 1`

## Errors

Symbol | Use
---|---
`!` | Before an expression, raise it as an error; after a function's return type, list the errors it raises; after a call, followed by `where`, handle its errors

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
`E` | Between the digits of a number and those of its exponent, as in `1E5` or `1.5E3`
`\|` | Within set, list and map builder notation "such that"
`{` | Within a string, start of an interpolated expression
`}` | Within a string, end of an interpolated expression
`\` | Within a string, keeps the next `"`, `{` or `}` from ending the string or starting or ending an interpolation

## Other Operators

Symbol | Use
---|---
`..`  | Exclusive range, or range step, and in `new(..)` one or more class arguments that are not written out
`..=` | Inclusive range

## Strings

Symbol | Use
---|---
`"`   | Start and end of a string
`"""` | Start and end of a docstring

## Comments

Symbol | Use
---|---
`#`  | Start of a comment, which runs to the end of the line

## Whitespace, identifiers and numbers

A space separates tokens, and a newline ends a statement.
The lexer reads `\r\n` as a newline, and rejects any other whitespace, such as a tab.
An identifier starts with an ASCII letter or `_`, followed by any number of ASCII letters, digits and `_`.
A number starts with an ASCII digit, see `number` in the [grammar](grammar.md).
Any other character outside a string or a comment is an error.
