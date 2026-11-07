<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Parse

Converts a `String` to an `AST`.
Each `AST` holds a `Node` and the `Position` it came from, which error messages point at.

## Lex

`lex/` turns the source into a `Vec<Token>`.
It errors on an illegal character.

## Parser

The parser walks the tokens through a `TokenIterator` and picks a parse function based on the next token.
The iterator's helpers eat an expected token, or call a parse function if or while the next token matches.
It errors if the tokens do not match the grammar.
