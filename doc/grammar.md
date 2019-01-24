# Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

    import           ::= "from" string [ ( "use" { id { "," id } | "useall" ) ] [ "as" id ] 
    
    file             ::= { module }
    module           ::= interface | util | class | script
    interface        ::= "type" id [ "[" id { "," id } "]" ] newline { newline }
                         { ( function-def | definition ) newline { newline } } ]
    util             ::= { import newline } newline { newline } 
                         "util" [ "[" id_maybe_type { "," id_maybe_type } "]" ] id [ "isa" id { "," id } ] 
                         newline { newline }
                         { top-level-def newline } { newline }
                         { ( immutable-declaration | function-def-bod ) newline { newline } }
    class            ::= { import newline } newline { newline }
                         [ util ]
                         "class" [ "[" id_maybe_type { "," id_maybe_type } "]"] [ constructor-args ] 
                         [ "isa" id { "," id } ] newline { newline } 
                         { top-level-def newline } { newline }
                         { ( constructor-def | function-def-bod | definition ) ) newline 
                         { newline } }
    script           ::= { import newline } { newline } 
                         { function-def newline { newline } } 
                         [ block-no-indent ]
    
    definition-call  ::= ( "self" | id  [ "." ] ) [ "?" ] id ( tuple | id )
    function-call    ::= [ id "::" ] id ( tuple | id )
    
    constructor-def  ::= "init" constructor-args [ "<-" expr-or-stmt ]
    constructor-args ::= "(" [ constructor-arg { "," constructor-arg } ] ")"
    constructor-arg  ::= [ "self" ] id-maybe-type
    
    function-def     ::= "def" [ "private" ] id "(" [ id-and-type { "," id-and-type } ] ")" [ ":" type ] [ raises ] 
    function-def-bod ::= function-def "->" expr-or-stmt
    function-def-anon::= tuple [ ":" type ] "->" expression
    operator-def     ::= "def" overridable-op "(" [ id-and-type ] ")" [ ":" type ] "->" expression
    
    id               ::= ( letter | "_" ) { ( letter | number | "_" ) }
    type             ::= id | type-tuple [ "->" type ]
    type-def         ::= "type" id "<-" type
    type-tuple       ::= "(" [ id-maybe-type { "," id-maybe-type } ] ")" 
    id-maybe-type    ::= ( id | type-tuple ) [ ":" type ]
    id-and-type      ::= ( id | type-tuple ) ":" type
    
    block            ::= indent { expr-or-stmt { newline } } dedent
    block-no-inent   ::= { expr-or-stmt { newline } }
    
    expr-or-stmt     ::= statement [ "if" maybe_expr ] [ ( raises | handle ) ]
                      | expression [ "if" maybe_expr ] [ ( raises | handle ) ]
                      | newline block
    statement        ::= ( "print" | "println" ) expression 
                      | definition 
                      | control-flow-stmt
                      | type-def
    expression       ::= "return" [ expression ]
                      | [ "self" ] expression
                      | expression "?or" expression
                      | collection 
                      | function-def-anon
                      | control-flow-expr 
                      | reassignment
                      | function-call 
                      | function-call-dir 
                      | operation
                      | sizeof
                      | "_"
                      
    raises           ::= "raises" "[" id { "," id } "]"
    handle           ::= "handle" "when" newline when-cases
    
    collection       ::= tupe | set | list | map
    tuple            ::= "(" zero-or-more-expr ")"
    set              ::= "{" zero-or-more-expr "}" | set-builder
    set-builder      ::= "{" expression "|" expression { "," expression } "}"
    list             ::= "[" zero-or-more-expr "]" | list-builder
    list-builder     ::= "[" expression "|" expression { "," expression } "]"
    map              ::= "{" expression "->" expression { "," expression "->" expression } "}" | map-builder
    map-builder      ::= "{" expression "->" expression "|" expression { "," expression } "}"
    zero-or-more-expr::= [ ( expression { "," expression } ]
    
    reassignment     ::= expression "<-" expression
    top-level-def    ::= definition [ forward ]
    forward          ::= "forward" id { "," id }
    definition       ::=  empty-def [ "<-" expression ]
    empty-def        ::= "def" [ "private" ] [ "mut" ] id-maybe-type [ "ofmut" ]

    operation        ::= relation | relation ( equality | instance-eq | binary-logic ) relation
    relation         ::= arithmetic [ comparison relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= inner-term [ multiclative term ]
    inner-term       ::= factor [ power inner-term ]
    factor           ::= [ unary ] ( constant | id | expression | sizeof )
    sizeof           ::= "|" expression "|"
    
    overrideable-op  ::= additive | "sqrt" | multiplicative | power | "eq" | comparison
    unary            ::= "not" | "sqrt" | additive
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/"
    power            ::= "^" | "mod"
    instance-eq      ::= "is" | "isnt" | "isa"
    equality         ::= "eq" | "neq"
    comparison       ::= "<=" | ">=" | "<" | ">"
    binary-logic     ::= "and" | "or"
    
    constant         ::= number | boolean | string
    number           ::= real | integer | e-notation
    real             ::= digit "." { digit }
    integer          ::= { digit }
    e-notation       ::= ( integer | real ) ( "e" | "E" ) [ "-" ] integer
    boolean          ::= "true" | "false"
    string           ::= """ { character } """
                                     
    control-flow-expr::= if | from | when
    if               ::= "if" expression "then" expr-or-stmt [ "else" expr-or-stmt ]
    when             ::= "when" expression newline when-cases
    when-cases       ::= indent { when-case { newline } } dedent
    when-case        ::= expression "->" expr-or-stmt
    
    control-flow-stmt::= while | foreach | "break" | "continue"
    while            ::= "while" expression "do" expr-or-stmt
    foreach          ::= "foreach" expression "in" expression "do" expr-or-stmt
    
    newline          ::= \n | \r\n
    comment          ::= "#" { character }

The language uses indentation to denote blocks. The indentation amount can't be described in the grammar directly, 
but it does adhere to the following rules:

* Every new expression or statement in a block must be preceded by n + 1 `indent`'s, where n is the amount of 
  `indent`'s before the block
* The same holds for every new `when-case` in a `when`

A `expression` is used in a situation where an expression is required, but we cannot know in advance whether it will be
an expression or statement without type checking the program.
`expr-or-stmt` may be used when it does not matter whether it is an expression or statement.
               