⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.1 Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

- ```( a | b | ... )``` = a or b or ...
- ```[ ... ]``` = zero or one
- ```{ ... }``` = zero or more

```ebnf
    file             ::= { expr-or-stmt }
    import           ::= [ "from" id ] "import" 
                         ( import-as | "{" import-as "," "}" | "{" import-as "," import-as { "," import-as } "}" )
    import-as        ::= id ( "as" id )

    type-def         ::= "type" type-not-fun ":" type-not-fun "when" ( expression | code-set )
    trait-def        ::= "trait" type-not-fun ( ":" type-not-fun { "," type-not-fun } ) [ ":=" statement ]
    class-def        ::= "class" type-not-fun [ fun-args ] [ ":" type-not-fun ] [ ":=" statement ]
    
    id               ::= { character }
    id-maybe-type    ::= id [ ":" type ]

    type-not-fun     ::= id [ generics ]
    type             ::= id [ generics ] [ "->" type ]
    generics         ::= "[" id { "," id } "]"
    
    expr-or-stmt     ::= ( statement | expression )
    statement        ::= control-flow-stmt
                      | definition
                      | reassignment
                      | type-def
                      | trait-def
                      | class-def
                      | import
                      | "return" [ expression ]
                      | code-set
    expression       ::= control-flow-expr 
                      | collection
                      | operation
                      | anon-fun
                      | call
                      | "_"
                      | code-block
                     
    reassignment     ::= expression ( ":=" | "+=" | "-=" | "*=" | "/=" | "^=" ) expression
    call             ::= expression [ ( "." | "?." ) ] id tuple [ "!" match-cases [ recover expression ] ]
    raise            ::= "!" id { "," id }
    
    # for all collections, we require one comma at least to avoid ambiguity
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
    map-builder      ::= "{ expression "=>" expression | expression { "," [ newline ] expression } }
      
    slice            ::= expression ( "::" | "::=" ) expression
    range            ::= expression ( ".." | "..=" ) expression
    
    definition       ::= variable-def | fun-def | type-def | trait-def | class-def

    variable-def     ::= "def" [ "fin" ] ( id-maybe-type | collection ) [ ":=" expression ]
    # type checker should check for valid combination of meta, total, pure
    fun-def          ::= "def" [ "meta" ] [ "total" ] [ "pure" ]
                         ( id | overridable-op ) fun-args [ "->" type ] [ raise ] 
                         [ ":=" expression ]
    fun-args         ::= "(" [ fun-arg ] { "," fun-arg } ")"
    fun-arg          ::= id-maybe-type [ ":=" expression ]
    anon-fun         ::= "\" [ id-maybe-type { "," id-maybe-type } ] ":=" expression
    
    operation        ::= relation [ ( equality | boolean-logic ) relation ]
    relation         ::= arithmetic [ comparison relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= inner-term [ ( multiplicative | range | slice ) term ]
    inner-term       ::= factor [ power inner-term ]
    factor           ::= [ unary ] ( literal | id | expression )
    
    overridable-op   ::= additive | multiplicative | power | "=" | "<" | ">"
    unary            ::= "not" | additive 
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/"
    power            ::= "^" | "mod"
    equality         ::= "=" | "!="
    comparison       ::= "<=" | ">=" | "<" | ">"
    boolean-logic    ::= "and" | "or"
    
    literal          ::= number | string
    number           ::= real | integer | e-notation
    real             ::= integer "." integer | "." integer | integer "."
    integer          ::= { digit }
    e-notation       ::= ( integer | real ) "E" [ "-" ] integer
    string           ::= """ { character } """
    
    code-block       ::= "[" expr-or-stmt { newline expr-or-stmt } "]" 
    code-set         ::= "{" expr-or-stmt { newline expr-or-stmt } "}"
    
    control-flow-expr::= if | match
    if               ::= "if" expression "then" expression [ "else" expression ]
    match            ::= "match" expression "with" map
    
    control-flow-stmt::= while | foreach | "break" | "continue"
    while            ::= "while" expression "do" expression
    foreach          ::= "for" expression "in" expression "do" expression
    
    newline          ::= <platform dependent>
```
