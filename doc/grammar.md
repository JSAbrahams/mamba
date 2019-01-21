# Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

    import           ::= "from" string [ ( "use" { id { "," id } | "useall" ) ] [ "as" id ] 
    
    file             ::= { module }
    module           ::= interface | util | class | script
    interface        ::= "type" id [ "[" id { "," id } "]" ] newline { newline }
                         { ( function-def | immutable-def | immutable-asssign | empty-def-type ) newline { newline } } ]
    util             ::= { import newline } newline { newline } 
                         "util" [ "[" id_maybe_type { "," id_maybe_type } "]" ] id [ "isa" id { "," id } ] 
                         newline { newline }
                         { im-defer-def newline } { newline }
                         { ( immutable-declaration | function-def-bod ) newline { newline } }
    class            ::= { import newline } newline { newline }
                         [ util ]
                         "class" [ "[" id_maybe_type { "," id_maybe_type } "]"] [ constructor-args ] 
                         [ "isa" id { "," id } ] newline { newline } 
                         { defer-def newline } { newline }
                         { ( constructor-def | function-def-bod | definition ) ) newline 
                         { newline } }
    script           ::= { import newline } { newline } 
                         { function-def newline { newline } } 
                         [ block ]
    
    definition-call  ::= [ "self" | id "." ] [ "?" ] id ( tuple | id )
    function-call    ::= [ id "::" ] id ( tuple | id )
    
    constructor-def  ::= "init" constructor-args [ "<-" expr-or-stmt ]
    constructor-args ::= "(" [ constructor-arg { "," constructor-arg } ] ")"
    constructor-arg  ::= [ "self" ] id-maybe-type
    
    function-def     ::= "def" [ "private" ] id "(" [ id-and-type { "," id-and-type } ] ")" [ ":" type ]
    function-def-bod ::= function-def "<-" expr-or-stmt
    function-def-anon::= tuple [ ":" type ] "->" expression
    operator-def     ::= "def" overridable-op "(" [ id-and-type ] ")" [ ":" type ] "<-" expression
    
    id               ::= ( letter | "_" ) { ( letter | number | "_" ) }
    type             ::= id | type-tuple [ "->" type ]
    type-def         ::= "type" id "<-" type
    type-tuple       ::= "(" [ id-maybe-type { "," id-maybe-type } ] ")" 
    id-maybe-type    ::= ( id | type-tuple ) [ ":" type ]
    id-and-type      ::= ( id | type-tuple ) ":" type
    
    block            ::= indent { expr-or-stmt newline } dedent
    
    expr-or-stmt     ::= statement 
                      | expression [ "if" maybe_expr ]
    statement        ::= ( "print" | "println" ) expression 
                      | definition 
                      | control-flow-stmt
                      | type-def
    expression       ::= "return" [ expression ] 
                      | expression "?or" expression
                      | instance
                      | operation 
                      | tuple 
                      | function-def-anon
                      | control-flow-expr 
                      | reassignment 
                      | function-call 
                      | function-call-dir 
                      | [ newline ] block
                      | [ "self" ] id
                      | collection
                      | sizeof
                      | "_"
    
    collection       ::= tupe | set | list | map
    tuple            ::= "(" zero-or-more-expr ")"
    set              ::= "{" zero-or-more-expr "}" | set-builder
    set-builder      ::= "{" expression | expression { "," expression } "}"
    list             ::= "[" zero-or-more-expr "]"
    map              ::= "{" expression "->" expression { "," expression "->" expression } "}"
    
    sizeof           ::= "|" expression "|"
    zero-or-more-expr::= [ ( expression { "," expression } ]
    
    reassignment     ::= expression "<-" expression
    defer-def        ::= definition [ "forward" id { "," id } ]
    im-defer-def     ::= immutable-def [ "forward" id { "," id } ]
    definition       ::= mutable-def | immutable-def
    mutable-def      ::= empty-def [ "<-" expression ]
    immutable-def    ::= empty-def [ "<-" expression ]
    empty-def        ::= "def" [ "private" ] [ "mut" ] id-maybe-type [ "ofmut" ] 
    empty-def-type   ::= "def" [ "mut" ] [ "ofmut" ] id-and-type

    operation        ::= relation | relation ( equality | instance-eq | binary-logic ) relation
    relation         ::= arithmetic [ comparison relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= inner-term [ multiclative term ]
    inner-term       ::= factor [ power inner-term ]
    factor           ::= [ additive | "sqrt" ] ( constant | id | expression )
    
    overrideable-op  ::= additive | "sqrt" | multiplicative | power | "eq" | comparison
    unary            ::= "not" | additive
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
    when             ::= "when" expression newline indent { when-case newline } dedent
    when-case        ::= expression "then" expr-or-stmt
    
    control-flow-stmt::= while | for | "break" | "continue"
    while            ::= "while" expression "do" expr-or-stmt
    for              ::= "for" expression "in" expression "do" expr-or-stmt
    
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
               