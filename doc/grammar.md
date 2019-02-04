# Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

    import           ::= "from" string [ ( "use" { id { "," id } | "useall" ) ] [ "as" id ]
    
    file             ::= { module }
    module           ::= interface | util | class | script
    interface        ::= "type" id [ "[" id_maybe_type { "," id_maybe_type } "]" ] newline { newline }
                         { ( function-def | definition ) newline { newline } } ]
    util             ::= { import newline } newline { newline }
                         "util" [ "[" id_maybe_type { "," id_maybe_type } "]" ] id [ "isa" id { "," id } ]
                         newline { newline }
                         { ( definition ) newline { newline } }
    class            ::= { import newline } newline { newline }
                         [ util ]
                         "class" [ "[" id_maybe_type { "," id_maybe_type } "]"] [ constructor-args ]
                         [ "isa" id { "," id } ] newline { newline }
                         { top-level-def newline } { newline }
                         { ( constructor | definition ) ) newline
                         { newline } }
    script           ::= { import newline } { newline }
                         { function-def newline { newline } }
                         [ block-no-indent ]
        
    constructor      ::= "init" constructor-args [ "<-" expr-or-stmt ]
    constructor-args ::= "(" [ constructor-arg { "," constructor-arg } ] ")"
    constructor-arg  ::= [ ( "vararg" | "def" ) ] id-and-type
    
    id               ::= ( letter | "_" ) { ( letter | number | "_" ) }
    type             ::= id | type-tuple [ "->" type ]
    type-tuple       ::= "(" [ id-maybe-type { "," id-maybe-type } ] ")" 
    type-def         ::= "type" id "isa" type [ post-when ]
    id-maybe-type    ::= id [ ":" type ]
    id-and-type      ::= id ":" type
    
    block            ::= indent { expr-or-stmt { newline } } dedent
    block-no-inent   ::= { expr-or-stmt { newline } }
    
    expr-or-stmt     ::= statement [ ( "if" maybe_expr | raises | handle ) ]
                      | expression [ ( "if" maybe_expr | raises | handle ) ]
    statement        ::= ( "print" | "println" ) expression 
                      | statement [ ( raises | handle ) ]
                      | control-flow-stmt
                      | definition
                      | type-def
                      | "retry"
    expression       ::= newline block
                      | expression ( "to" | "toincl" ) expression
                      | "(" expression ")" [ "->" expression ]
                      | expression [ ( raises | handle ) ]
                      | "return" [ expression ]
                      | expression "?or" expression
                      | expression "as" id
                      | function-def-anon
                      | control-flow-expr 
                      | reassignment
                      | collection
                      | operation
                      | call
                      | "_"
                     
    call             ::= function-call | method-call
    function-call    ::= expression [ "::" expression ] ( expression | "(" [ expression { "," expression} ] ")" )
    method-call      ::= expression "." ( expression | "(" [ expression { "," expression} ] ")" ) [ "?" ]
                    
    post-when        ::= "when" newline indent { condition } dedent
    condition        ::= expression "else" expression
    conditions       ::= indent { condition } dedent
    condition        ::= expression "else" expression
    raises           ::= "raises" "[" id { "," id } "]"
    handle           ::= "handle" "when" newline when-cases
    
    collection       ::= tuple | set | list | map
    tuple            ::= "(" zero-or-more-expr ")"
    set              ::= "{" zero-or-more-expr "}" | set-builder
    set-builder      ::= "{" expression "|" expression { "," expression } "}"
    list             ::= "[" zero-or-more-expr "]" | list-builder
    list-builder     ::= "[" expression "|" expression { "," expression } "]"
    map              ::= "{" expression "->" expression { "," expression "->" expression } "}" | map-builder
    map-builder      ::= "{" expression "->" expression "|" zero-or-more-expr "}"
    zero-or-more-expr::= [ ( expression { "," expression } ]
    
    reassignment     ::= expression "<-" expression
    
    definition       ::= "def" ( [ "private" ] ( variable-def | fun-def ) | operator-def )
    variable-def     ::= [ "mut" ] id-maybe-type [ "<-" expression [ ( "when" newline conditions | forward ) ] ]
    operator-def     ::= overridable-op [ "(" [ id-and-type ] ")" ] ":" type [ "->" expression ]
    fun-def          ::= id "(" [ fun-args ] ")" ":" type [ raises ] [ "->" expression ]
    fun-args         ::= fun-arg { "," fun-arg }
    fun-arg          ::= [ "vararg" ] id-and-type
    forward          ::= "forward" id { "," id }

    operation        ::= relation | relation ( equality | instance-eq | binary-logic ) relation
    relation         ::= arithmetic [ comparison relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= inner-term [ multiclative term ]
    inner-term       ::= factor [ power inner-term ]
    factor           ::= [ unary ] ( literal | id | expression )
    
    overrideable-op  ::= additive | "sqrt" | multiplicative | power | "eq" | "<" | ">"
    unary            ::= "not" | "sqrt" | additive 
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/"
    power            ::= "^" | "mod"
    instance-eq      ::= "is" | "isnt" | "isa" | "isnta" | "in"
    equality         ::= "eq" | "neq"
    comparison       ::= "<=" | ">=" | "<" | ">"
    binary-logic     ::= "and" | "or"
    
    literal          ::= number | boolean | string
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

The language uses indentation to denote code blocks. The indentation amount can't be described in the grammar directly, 
but it does adhere to the following rules:

* Every new expression or statement in a block must be preceded by n + 1 `indent`'s, where n is the amount of 
  `indent`'s before the block
* The same holds for every new `when-case` in a `when`

A `expression` is used in a situation where an expression is required. However we cannot always know in advance whether
this is the case, e.g. when it is a function call. In This should be verified by the type checker.
`expr-or-stmt` may be used when it does not matter whether something is an expression or statement, such as the body of
a loop.
               