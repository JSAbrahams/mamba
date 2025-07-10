⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.1 Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

- ```( a | b | ... )``` = a or b or ...
- ```[ ... ]``` = zero or one
- ```{ ... }``` = zero or more

```ebnf
    file             ::= { expr-or-stmt }
    import           ::= [ "from" id ] "import" id { "," id } [ as id { "," id } ]

    type-def         ::= "type" type ":" type ( ":=" "{" code-set "}" | "when" [ code-set ] )
    trait-def        ::= "trait" type ( ":" type { "," type } ) [ ":=" code-set ]
    type-tuple       ::= "(" [ type ] { "," type } ")"
    
    class            ::= "class" id [ fun-args ] [ ":" ( type | type-tuple ) ] [ ":=" code-set ]
    generics         ::= "[" id { "," id } "]"
    
    id               ::= { character }
    id-maybe-type    ::= id [ ":" type ]

    type             ::= ( id [ generics ] | type-tuple ) [ "->" type ]
    type-tuple       ::= "(" [ type { "," type } ] ")"
    
    expr-or-stmt     ::= ( statement | expression )
    statement        ::= control-flow-stmt
                      | definition
                      | reassignment
                      | type-def
                      | "pass"
                      | class
                      | type-def
                      | import
    expression       ::= "(" expression ")"
                      | expression "?or" expression
                      | "return" [ expression ]
                      | expression "as" id 
                      | control-flow-expr 
                      | code-block
                      | collection
                      | key-value
                      | operation
                      | anon-fun
                      | call
                      | "_"
                     
    reassignment     ::= expression ( ":=" | "+=" | "-=" | "*=" | "/=" | "^=" | ">>=" | "<<=" ) code-block
    call             ::= code-block [ ( "." | "?." ) ] id tuple [ "!" match-cases [ recover code-block ] ]
    raise            ::= "!" id { "," id }
    
    collection       ::= tuple | set | list | map
    tuple            ::= "(" code-block { "," code-block } ")"
    set              ::= "{" code-block { "," code-block } "}" | set-builder
    set-builder      ::= "{" expression "|" expression { "," expression } "}"
    list             ::= "[" code-block { "," code-block } "]" | list-builder
    list-builder     ::= "[" expression "|" expression { "," expression } "]"
    
    slice            ::= code-block ( "::" | "::=" ) code-block
    range            ::= code-block ( ".." | "..=" ) code-block
    
    definition       ::= "def" ( variable-def | fun-def ) | type-def | trait-def | class-def

    variable-def     ::= [ "fin" ] ( id-maybe-type | collection ) [ ":=" code-block ] [ forward ]
    fun-def          ::= ( [ "const" ] | [ "total" ] [ "pure" ] ) ( id | overridable-op ) fun-args [ "->" type ] [ raise ] [ ":=" code-block ]
    fun-args         ::= "(" [ fun-arg ] { "," fun-arg } ")"
    fun-arg          ::= id-maybe-type [ ":=" code-block ]
    anon-fun         ::= "\" [ id-maybe-type { "," id-maybe-type } ] ":=" code-block
    
    operation        ::= relation [ ( equality | instance-eq | boolean-logic ) relation ]
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
    instance-eq      ::= "is" | "isa"
    equality         ::= "=" | "!="
    comparison       ::= "<=" | ">=" | "<" | ">"
    boolean-logic    ::= "and" | "or"
    
    literal          ::= number | string | "None"
    number           ::= real | integer | e-notation
    real             ::= integer "." integer | "." integer | integer "."
    integer          ::= { digit }
    e-notation       ::= ( integer | real ) "E" [ "-" ] integer
    string           ::= """ { character } """
    
    code-block       ::= expr-or-stmt
                      | "[" expr-or-stmt "]" 
                      | "[" newline expr-or-stmt { newline expr-or-stmt } "]"
    code-set         ::= expr-or-stmt
                      | "{" expr-or-stmt "}"
                      | "{" newline expr-or-stmt { newline expr-or-stmt } "}"
    
    control-flow-expr::= if | match
    if               ::= "if" code-block "then" code-block [ "else" code-block ]
    match            ::= "match" code-block "with" match-cases
    match-cases      ::= "{" match-case "}" | "{" newline match-case { newline match-case } "}"
    match-case       ::= expression "=>" code-block
    
    control-flow-stmt::= while | foreach | "break" | "continue"
    while            ::= "while" code-block "do" code-block
    foreach          ::= "for" code-block "in" code-block "do" code-block
    
    newline          ::= <platform dependent>
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
