⬅ [🏠 Home](../README.md)

⬅ [3 📚 Specification](README.md)

# 3.1 Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

- ```( a | b | ... )``` = a or b or ...
- ```[ ... ]``` = zero or one
- ```{ ... }``` = zero or more

```ebnf
    file             ::= { expr-or-stmt }
    # the aliases are one list rather than one per name, so they are positional
    import           ::= [ "from" id ] "import" id { "," id } [ "as" id { "," id } ]

    # a class may list several parents; a trait taking more than one parent is future work
    trait-def        ::= "trait" type-not-fun [ ":" parent ] [ code-set ]
    # a class's arguments are its fields; there is no constructor to declare
    # its body holds only field and method declarations, plus a leading docstring
    class-def        ::= "class" type-not-fun [ fun-args ] [ ":" parent { "," parent } ] [ class-set ]
    parent           ::= type-not-fun [ "(" [ expression { "," expression } ] ")" ]
    class-set        ::= "where" [ docstring newline ] class-member { newline class-member } "end"
    class-member     ::= variable-def | fun-def | class-new
    # asserts something about the "new" the class already has, rather than declaring one
    # its list stands for the class arguments: "()" none, "(_)" one, "(..)" one or more
    class-new        ::= "def" [ "pure" ] "new" "(" [ "_" | ".." ] ")"
    
    id               ::= { character }
    # "mut" marks one binding, so it may only precede an id, never a tuple
    # an id-tuple therefore takes one marker per element, at every depth
    id-tuple         ::= "(" id-element { "," id-element } ")"
    id-element       ::= [ "mut" ] id | id-tuple
    id-maybe-type    ::= ( [ "mut" ] id | id-tuple ) [ ":" type ]

    # "Self" is a type naming the enclosing class, usable anywhere in a class body
    # a trailing "?" makes a type nullable; a type set is a union
    type-not-fun     ::= ( id [ generics ] | "Self" | type-tuple | type-set ) [ "?" ]
    type             ::= type-not-fun [ "->" type ]
    type-tuple       ::= "(" [ type { "," type } ] ")"
    type-set         ::= "{" [ type { "," type } ] "}"
    generics         ::= "[" generic { "," generic } "]"
    # the optional id bounds the parameter, as in `[T: Num]`
    generic          ::= id [ ":" id ]
    
    expr-or-stmt     ::= ( statement | expression )
    statement        ::= control-flow-stmt
                      | definition
                      | reassignment
                      | type-def
                      | trait-def
                      | class-def
                      | import
                      | using-block
                      | "return" [ expression ]
                      | code-set
    expression       ::= control-flow-expr 
                      | collection
                      | operation
                      | anon-fun
                      | call
                      | index
                      | "_"
                      | ".."
                      | code-block
                     
    reassignment     ::= expression ( ":=" | "+=" | "-=" | "*=" | "/=" | "^=" ) expression
    call             ::= expression [ ( "." | "?." ) ] id tuple [ "!" match-cases ]
    # round brackets call, square brackets index
    index            ::= expression "[" expression "]"
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

    # type-def is a type alias / refinement (`type ... when ...`). It is future work: it has no
    # production below, and the "type" keyword is not accepted by the parser yet.

    # a binding is immutable unless marked "mut", which id-maybe-type carries for
    # arguments and "self" as well as for variables
    # destructuring a list or set, as in `def [a, b]`, is future work and does not parse
    variable-def     ::= "def" id-maybe-type [ ":=" expression ]
    # type checker should check for valid combination of meta, total, pure
    # the "meta" and "total" modifiers are future work; only "pure" is implemented
    fun-def          ::= "def" [ "meta" ] [ "total" ] [ "pure" ]
                         ( id | overridable-op ) fun-args [ "->" type ] [ raise ] 
                         [ ":=" expression ]
    # a fun-def with no "self" argument is an associated function, called on the class
    # rather than on an instance; "new" is one of these
    fun-args         ::= "(" [ fun-arg ] { "," fun-arg } ")"
    # a placeholder binds nothing, so it takes neither a type nor a default
    # the grammar lets one stand wherever an expression or an argument does, and the type
    # checker narrows it to "class-new" above; a match case is where ".." goes next
    fun-arg          ::= id-maybe-type [ ":=" expression ] | "_" | ".."
    anon-fun         ::= "\" [ id-maybe-type { "," id-maybe-type } ] ":=" expression
    
    # loosest binding first; each level is right-recursive into itself
    operation        ::= relation [ boolean-logic operation ]
    relation         ::= arithmetic [ ( comparison | equality | "in" ) relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= unary [ ( multiplicative | range | slice ) term ]
    unary            ::= [ prefix ] inner-term
    inner-term       ::= factor [ power inner-term ]
    factor           ::= literal | id | expression
    
    overridable-op   ::= additive | multiplicative | power | "=" | "<" | ">"
    prefix           ::= "not" | "sqrt" | additive
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/" | "//" | "mod"
    power            ::= "^"
    equality         ::= "=" | "!="
    comparison       ::= "<=" | ">=" | "<" | ">"
    # "?" is the null-coalescing operator: `a ? b` is `a` unless it is None, then `b`
    boolean-logic    ::= "and" | "or" | "?"
    
    literal          ::= number | string
    number           ::= real | integer | e-notation
    real             ::= integer "." integer | "." integer | integer "."
    integer          ::= { digit }
    e-notation       ::= ( integer | real ) "E" [ "-" ] integer
    string           ::= """ { character } """
    
    code-block       ::= "do" expr-or-stmt { newline expr-or-stmt } "end" 
    code-set         ::= "where" expr-or-stmt { newline expr-or-stmt } "end"
    using-block      ::= "using" expression [ "as" id-maybe-type ] code-block
    
    control-flow-expr::= if | match
    # a branch is one expr-or-stmt, so it may be a bare statement or a "do ... end" block
    if               ::= "if" expression "then" expr-or-stmt [ "else" expr-or-stmt ]
    match            ::= "match" expression match-cases
    match-cases      ::= "where" { match-case } "end"
    # the type annotation names the error a handle case catches, as in `err: MyErr => ...`
    match-case       ::= [ "mut" ] expression [ ":" type ] "=>" expr-or-stmt
    
    control-flow-stmt::= while | foreach | "break" | "continue"
    while            ::= "while" expression code-block
    # the loop variable binds, so it is an id or a tuple of them, not any expression
    foreach          ::= "for" ( id | id-tuple ) "in" expression code-block
    
    newline          ::= <platform dependent>
```
