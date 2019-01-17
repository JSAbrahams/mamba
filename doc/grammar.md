# Grammar

The grammar of the language in Extended Backus-Naur Form (EBNF).

    import           ::= "use" string [ "as" id ] [ ( "use" { id { "," id } | "useall" ) ]
    
    module           ::= interface | util | class | script
    interface        ::= "type" id newline { newline }
                         { ( function-def | immutable-def | immutable-asssign ) newline { newline } } ]
    util             ::= { import newline } newline { newline } 
                         "util" [ "[" id { "," id } "]" ] id [ "isa" id { "," id } ] newline { newline }
                         { im-defer-def newline } { newline }
                         { [ "private" ] ( immutable-declaration | function-def-bod ) newline { newline } }
    class            ::= { import newline } newline { newline }
                         [ util ]
                         "class" [ constructor-args ] [ "isa" id { "," id } ] newline { newline } 
                         { defer-def newline } { newline }
                         { ( constructor-def | [ "private" ] ( function-def-bod | declaration ) ) newline { newline } }
    script           ::= { import newline } { newline } 
                         { function-def newline { newline } } 
                         [ do-block ]
    
    method-call      ::= [ "self" | id "." ] id tuple
    function-call    ::= ( [ "self" | id "::" ] id | function-anon ) tuple
    
    constructor-def  ::= "init" constructor-args [ "<-" expr-or-stmt ]
    constructor-args ::= "(" [ constructor-arg { "," constructor-arg } ] ")"
    constructor-arg  ::= [ "self" ] function-arg
    
    function-def     ::= "def" id "(" [ function-arg { "," function-arg } ] ")" [ ":" type ]
    function-def-bod ::= function-def "<-" expr-or-stmt
    function-arg     ::= id ":" type
    
    function-anon    ::= args-anon "<-" maybe-expr
    args-anon        ::= id | "(" [ args-anon { "," args-anon } ] ")"
    
    type             ::= id | type "<-" type | "(" [ type { "," type } ] ")"
    type-def         ::= "type" id "<-" type
    
    block            ::= indent { expr-or-stmt newline } dedent
    
    expr-or-stmt     ::= statement 
                      | maybe-expr [ ( "if" | "unless" ) maybe_expr ]
    statement        ::= "print" maybe-expr 
                      | declaration 
                      | control-flow-stmt
                      | type-def
    maybe-expr       ::= "return" [ maybe-expr ] 
                      | instance
                      | operation 
                      | tuple 
                      | function-anon
                      | control-flow-expr 
                      | reassignment 
                      | function-call 
                      | function-call-dir 
                      | newline block
                      | set-builder
                      | "_"
    
    id               ::= [ "self" ] ( letter | "_" ) { ( letter | number | "_" ) }
    
    tuple            ::= "(" zero-or-more-expr ")"
    set              ::= "{" zero-or-more-expr "}"
    set-builder      ::= "{" maybe-expr | maybe-expr { "," maybe-expr } "}"
    list             ::= "[" zero-or-more-expr "]"
    zero-or-more-expr::= [ ( maybe-expr { "," maybe-expr } ]
    
    reassignment     ::= maybe-expr "<-" maybe-expr
    defer-def        ::= declaration [ "forward" id { "," id } ]
    im-defer-def     ::= immutable-declaration [ "forward" id { "," id } ]
    mutable-def      ::= "def" "mut" id [ ":" type ]
    immutable-def    ::= "def" id [ ":" type ]

    operation        ::= relation | relation ( equality | binary-logic ) relation
    relation         ::= arithmetic [ comparison relation ]
    arithmetic       ::= term [ additive arithmetic ]
    term             ::= inner-term [ multiclative term ]
    inner-term       ::= factor [ power inner-term ]
    factor           ::= [ additive | "sqrt" ] ( constant | id | maybe-expr )
    
    unary            ::= "not" | additive
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/"
    power            ::= "^" | "mod"
    equality         ::= "equals" | "is" | "notequals" | "isnot" | "isa"
    comparison       ::= "<=" | ">=" | "<" | ">"
    binary-logic     ::= "and" | "or"
    
    constant         ::= number | boolean | string
    number           ::= real | integer | e-notation
    real             ::= digit "." { digit } | "." digit { digit }
    integer          ::= { digit }
    e-notation       ::= ( integer | real ) ( "e" | "E" ) [ "-" ] integer
    boolean          ::= "true" | "false"
    string           ::= "\"" { character } "\""
                                     
    control-flow-expr::= if | from | when
    if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
    when             ::= "when" maybe-expr newline indent { when-case newline } dedent
    when-case        ::= maybe-expr "then" expr-or-stmt
    
    control-flow-stmt::= loop | while | for | "break" | "continue"
    while            ::= "while" maybe-expr "do" expr-or-stmt
    for              ::= "for" maybe-expr "in" maybe-expr "do" expr-or-stmt
    
    newline          ::= \n | \r\n

The language uses indentation to denote blocks. The indentation amount can't be described in the grammar directly, 
but it does adhere to the following rules:

* Every new expression or statement in a block must be preceded by n + 1 `indent`'s, where n is the amount of 
  `indent`'s before the block
* The same holds for every new `when-case` in a `when`

A `maybe-expr` is used in a situation where an expression is required,  but we cannot know in advance whether it will be
an expression or statement without type checking the program.
`expr-or-stmt` may be used when it does not matter whether it is an expression or statement.
               