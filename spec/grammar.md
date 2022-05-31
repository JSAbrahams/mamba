# Grammar

Below is the grammar specification of the language.

## Notation Of EBNF

The grammar of the language in Extended Backus-Naur Form (EBNF).

- ```( a | b | ... )``` = a or b or ...
- ```[ ... ]``` = zero or one
- ```{ ... }``` = zero or more

```
    file             ::= block
    import           ::= [ "from" id ] "import" id { "," id } [ as id { "," id } ]

    type-def         ::= "type" type [ ":" type ] ( newline block | "when" [ conditions ] )
    conditions       ::= ( newline indent { condition newline } dedent | condition )
    condition        ::= expression [ "else" expression ]
    type-tuple       ::= "(" [ type ] { "," type } ")"
    
    class            ::= "class" id [ fun-args ] [ ":" ( type | type-tuple ) ] ( newline block )
    generics         ::= "[" id { "," id } "]"
    
    id               ::= { character }
    id-maybe-type    ::= id [ ":" type ]

    type             ::= ( id [ generics ] | type-tuple ) [ "->" type ]
    type-tuple       ::= "(" [ type { "," type } ] ")"
    
    block            ::= indent { expr-or-stmt } dedent
    
    expr-or-stmt     ::= ( statement | expression [ ( raises | handle ) ] )
    statement        ::=  control-flow-stmt
                      | definition
                      | reassignment
                      | type-def
                      | "retry"
                      | "pass"
                      | class
                      | type-def
                      | comment
                      | import
    expression       ::= "(" expression ")"
                      | expression "?or" expression
                      | "return" [ expression ]
                      | expression "as" id 
                      | control-flow-expr 
                      | newline block
                      | collection
                      | index
                      | key-value
                      | operation
                      | anon-fun
                      | call
                      | "_"
                     
    reassignment     ::= expression ( ":=" | "+=" | "-=" | "*=" | "/=" | "^=" | ">>=" | "<<=" ) expression
    anon-fun         ::= "\" [ id-maybe-type { "," id-maybe-type } ] "=>" expression
    call             ::= expression [ ( "." | "?." ) ] id tuple
    
    raises           ::= "raise" id { "," id }
    handle           ::= "handle" newline match-cases
    
    collection       ::= tuple | set | list | map
    tuple            ::= "(" { expression } ")"
    set              ::= "{" { expression } "}" | set-builder
    set-builder      ::= "{" expression "|" expression { "," expression } "}"
    list             ::= "[" { expression } "]" | list-builder
    list-builder     ::= "[" expression "|" expression { "," expression } "]"
    
    slice            ::= expression ( "::" | "::=" ) expression [ "::" expression ]
    range            ::= expression ( ".." | "..=" ) expression [ ".." expression ]
    index            ::= expression "[" expression "]"
    
    definition       ::= "def" ( variable-def | fun-def | operator-def )

    variable-def     ::= [ "fin" ] ( id-maybe-type | collection ) [ ":=" expression ] [ forward ]
    operator-def     ::= [ "pure" ] overridable-op [ "(" [ id-maybe-type ] ")" ] "->" type 
                         [ "=>" ( expr-or-stmt | newline block ) ]
    fun-def          ::= [ "pure" ] id fun-args [ "->" type ] [ raises ] 
                         [ "=>" ( expr-or-stmt | newline block ) ]
    fun-args         ::= "(" [ fun-arg ] { "," fun-arg } ")"
    fun-arg          ::= [ "vararg" ] ( id-maybe-type | literal ) [ ":=" expression ]
    forward          ::= "forward" id { "," id }
    
    operation        ::= relation [ ( equality | instance-eq | binary-logic ) relation ]
    relation         ::= arithmetic [ comparison relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= inner-term [ ( multiclative | range | slice ) term ]
    inner-term       ::= factor [ power inner-term ]
    factor           ::= [ unary ] ( literal | id | expression )
    
    overrideable-op  ::= additive | multiplicative | power | "=" | "<" | ">"
    unary            ::= "not" | additive 
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/"
    power            ::= "^" | "mod"
    instance-eq      ::= "is" | "isnt" | "isa" | "isnta"
    equality         ::= "=" | "!="
    comparison       ::= "<=" | ">=" | "<" | ">"
    binary-logic     ::= "and" | "or"
    
    literal          ::= number | boolean | string | "None"
    number           ::= real | integer | e-notation
    real             ::= integer "." integer | "." integer | integer "."
    integer          ::= { digit }
    e-notation       ::= ( integer | real ) "E" [ "-" ] integer
    boolean          ::= "True" | "False"
    string           ::= """ { character } """
    
    newline-block    ::= newline block | expr-or-stmt
    one-or-more-expr ::= expression { "," expression }
    
    control-flow-expr::= if | match
    if               ::= "if" one-or-more-expr "then" newline-block [ "else" newline-block ]
    match            ::= "match" one-or-more-expr "with" newline match-cases
    match-cases      ::= indent { match-case { newline } } dedent
    match-case       ::= expression "=>" expr-or-stmt
    
    control-flow-stmt::= while | foreach | "break" | "continue"
    while            ::= "while" one-or-more-expr "do" newline-block
    foreach          ::= "for" one-or-more-expr "in" expression "do" newline-block
    
    newline          ::= newline-char [ comment ]
    newline-char     ::= \n | \r\n
    comment          ::= "#" { character } newline
```

## Notes

An `expression` is used in a situation where an expression is required.
This allows the parser to short-circuit if something is definitely not an expression where it should be.
However, we cannot always know in advance whether something is an expression, e.g. when it is a function call.
Those cases should be verified by the type checker.
An `expr-or-stmt` may be used when it does not matter whether something is an expression or statement, such as the body of a loop.

We do not systematically desugar multiple delimited by commas, or a single expression, to tuples, as is the case in Python.
This prevents ambiguity in the grammar as specified above, and also prevents confusing situations such as `(0)` and `0` being equal.
Instead, we only do this in specific contexts, such as in the conditional of control flows.
