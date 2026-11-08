⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.1 Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

- ```( a | b | ... )``` = a or b or ...
- ```[ ... ]``` = zero or one
- ```{ ... }``` = zero or more
- ```'"'``` = a terminal that holds a double quote, so it is in single quotes

```ebnf
    file             ::= { expr-or-stmt }
    import           ::= [ "from" id ] "import" id { "," id } [ "as" id { "," id } ]

    trait-def        ::= "trait" type-not-fun [ ":" parent ] [ code-set ]
    class-def        ::= "class" type-not-fun [ fun-args ] [ ":" parent { "," parent } ] [ class-set ]
    parent           ::= type-not-fun [ "(" [ expression { "," expression } ] ")" ]
    class-set        ::= "where" [ docstring newline ] class-member { newline class-member } "end"
    class-member     ::= variable-def | fun-def

    id               ::= { character }
    id-tuple         ::= "(" id-element { "," id-element } ")"
    id-element       ::= [ "mut" ] id | id-tuple
    id-maybe-type    ::= ( [ "mut" ] id | id-tuple ) [ ":" type ]

    type-not-fun     ::= ( id [ generics ] | type-tuple | type-set ) [ "?" ]
    type             ::= type-not-fun [ "->" type ]
    type-tuple       ::= "(" [ type { "," type } ] ")"
    type-set         ::= "{" [ type { "," type } ] "}"
    generics         ::= "[" generic { "," generic } "]"
    generic          ::= id [ ":" id ]

    expr-or-stmt     ::= ( statement | expression )
    statement        ::= control-flow-stmt
                      | definition
                      | reassignment
                      | import
                      | using-block
                      | "return" [ expression ]
                      | code-set
    expression       ::= control-flow-expr
                      | collection
                      | operation
                      | anon-fun
                      | call
                      | "_"
                      | ".."
                      | code-block

    reassignment     ::= expression ( ":=" | "+=" | "-=" | "*=" | "/=" | "^=" ) expression
    # a call with one argument also indexes, as in `a(0)`
    call             ::= expression [ ( "." | "?." ) ] id tuple [ "!" match-cases ]
    raise            ::= "!" id { "," id }

    collection       ::= tuple | set | set-builder | list | list-builder | map | map-builder
    tuple            ::= "(" "," ")" | "(" expression "," ")"
                      | "(" expression "," [ newline ] expression { "," [ newline ] expression } ")"
    set              ::= "{" "," "}" | "{" expression "," "}"
                      | "{" expression { "," [ newline ] expression } "}"
    set-builder      ::= "{" expression "|" expression { "," [ newline ] expression } "}"
    list             ::= "[" "," "]" | "[" expression "," "]"
                      | "[" expression { "," [ newline ] expression } "]"
    list-builder     ::= "[" expression "|" expression { "," [ newline ] expression } "]"
    map              ::= "{" expression "=>" expression "," "}"
                      | "{" expression "=>" expression { "," [ newline ] expression "=>" expression } "}"
    map-builder      ::= "{" expression "=>" expression "|" expression { "," [ newline ] expression } "}"

    range            ::= expression ( ".." | "..=" ) expression

    definition       ::= variable-def | fun-def | trait-def | class-def
    variable-def     ::= "let" id-maybe-type [ ":=" expression ]
    fun-def          ::= "def" [ "meta" ] [ "total" ] [ "pure" ]
                         ( id | overridable-op ) fun-args [ "->" type ] [ raise ]
                         [ ":=" expression ]
    fun-args         ::= "(" [ fun-arg ] { "," fun-arg } ")"
    fun-arg          ::= id-maybe-type [ ":=" expression ] | "_" | ".."
    anon-fun         ::= "\" [ id-maybe-type { "," id-maybe-type } ] ":=" expression

    # loosest binding first
    operation        ::= relation [ boolean-logic operation ]
    relation         ::= arithmetic [ ( comparison | equality | "in" ) relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= unary [ ( multiplicative | range ) term ]
    unary            ::= [ prefix ] inner-term
    inner-term       ::= factor [ power inner-term ]
    factor           ::= literal | id | expression

    overridable-op   ::= additive | multiplicative | power | "=" | "<" | ">"
    prefix           ::= "not" | additive
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/" | "//" | "mod"
    power            ::= "^"
    equality         ::= "=" | "!="
    comparison       ::= "<=" | ">=" | "<" | ">"
    # "?" is null-coalescing
    boolean-logic    ::= "and" | "or" | "?"

    literal          ::= number | string
    number           ::= real | integer | e-notation
    real             ::= integer "." { digit }
    integer          ::= digit { digit }
    e-notation       ::= ( integer | real ) "E" { digit }
    string           ::= '"' { character } '"'
    docstring        ::= '"""' { character } '"""'

    code-block       ::= "do" expr-or-stmt { newline expr-or-stmt } "end"
    code-set         ::= "where" expr-or-stmt { newline expr-or-stmt } "end"
    using-block      ::= "using" expression [ "as" id-maybe-type ] code-block

    control-flow-expr::= if | match
    if               ::= "if" expression "then" expr-or-stmt [ "else" expr-or-stmt ]
    match            ::= "match" expression match-cases
    match-cases      ::= "where" match-case { match-case } "end"
    match-case       ::= [ "mut" ] expression [ ":" type ] [ "if" expression ] "=>" expr-or-stmt

    control-flow-stmt::= while | foreach | "break" | "continue"
    while            ::= "while" expression code-block
    foreach          ::= "for" ( id | id-tuple ) "in" expression code-block

    newline          ::= <platform dependent>
```
