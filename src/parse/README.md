<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Parse

Converts a `String` to an Abstract Syntax Tree(AST).

Implementation wise, an `AST` is where a `Node` are stored.

- `AST` is container which stores a `Position` and a `Node`.
- `Node` is what is used to construct the language.

## Lex

The lexer converts a string to a vector of `Token`s. This is in part to simplify the parsing stage.

The lexer also deals with the logic necessary to detect indentations. It produces special tokens `Indent` and `Dedent`,
which the next stage uses to identify the start and end of blocks.

We get an error if we encounter an illegal character

## Parse

The parsing stage iterates over tokens and depending on the next token picks the relevant parsing function.

It takes as input a vector of tokens and produces a `TokenIterator`. This iterator has multiple internal methods which
make it easy to:

- Eat a token if we expect it, which also gives the position. It errors if it is not a token we expect.
- Conditionally call a parse method if the next token is a certain token, or form a set of tokens.
- Conditionally call a parse method while the next token is a certain token, or from a set of tokens.

The output is an AST. The `Position` within each `AST` also allows one to generate elegant error messages. They allow
the transpiler to print error messages where it points to where in the source something went wrong.

We get an error if the list of tokens does not conform to the language grammar.
