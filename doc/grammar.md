# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    module-import    ::= "from" id ( "use" id [ "as" id ] | "useall" )
    
    module           ::= type | util | class | script
    type             ::= { module-import newline } { newline } 
                         "type" id [ newline { newline }
                         { ( function-def | definition | immutable-asssign ) newline { newline } } ]
    util             ::= { module-import newline } { newline } 
                         "util" id [ newline [ "isa" id [ { "," id } ] ] [ newline { newline } 
                         { ( immutable-assign | function-def-bod ) newline { newline } } ]
    class            ::= { module-import newline } { newline } 
                         "class" id [ "isa" id [ { "," id } ] ] [ newline { newline } 
                         { ( "util" ( function-def-bod | immutable-assign ) | 
                             "private" ( function-def-bod | assignment ) ) newline { newline } } ]
    script           ::= { module-import newline } { newline } 
                         { function-def newline { newline } } 
                         [ do-block ]
    
    function-call    ::= maybe-expr "." id tuple
    function-call-dir::= id tuple
    function-def     ::= "fun" id "(" function-args ")" [ ":" function-type ]
    function-def-bod ::= function-def "->" expr-or-stmt
    function-args    ::= function-type ":" function-type [ "," function-args ]
    function-type    ::= id | static-tuple | function-tuple "->" function-type
    function-tuple   ::= "(" [ function-type { "," function-type } ] ")"
    function-anon    ::= function-tuple "->' maybe-expr
    
    do-block         ::= { { indent } expr-or-stmt newline [ { indent } newline ] }
    
    expr-or-stmt     ::= statement 
                      | maybe-expr [ ( "if" | "unless" ) maybe_expr ]
                      
    statement        ::= "print" maybe-expr 
                      | assignment 
                      | control-flow-stmt
    maybe-expr       ::= "return" [ maybe-expr ] 
                      | operation 
                      | tuple 
                      | control-flow-expr 
                      | reassignment 
                      | function-call 
                      | function-call-dir 
                      | newline do-block
    
    id               ::= { ( character | number | "_" ) }
    tuple            ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
    
    reassignment     ::= maybe-expr "<-" maybe-expr
    assignment       ::= mutable-assign | immutable-assign
    mutable-assign   ::= [ "mutable" ] immutable-assignment
    immutable-assign ::= variable-def "<-" maybe-expr
    definition       ::= "let" id

    operation        ::= arithmetic | arithmetic relational maybe-expr
    arithmetic       ::= term | unary arithmetic | term additive maybe-expr
    term             ::= factor | factor multiclative-operator maybe-expr
    factor           ::= constant | id
    
    unary            ::= "not" | additive
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/" | "^" | "mod"
    relational       ::= equality | comparison | binary-logic
    equality         ::= "equals" | "is" | "notequals" | "isnot"
    comparison       ::= "<=" | ">=" | "<" | ">"
    binary-logic     ::= "and" | "or"
    
    constant         ::= number | boolean | string
    number           ::= real | integer | e-notation
    real             ::= digit "." digit
    integer          ::= digit
    e-notation       ::= ( integer | real ) ( "e" | "E" ) [ "-" ] integer
    boolean          ::= "true" | "false"
    string           ::= "\"" { character } "\""
                                     
    control-flow-expr::= if | when | from
    if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
    from             ::= "from" maybe-expr [ newline ] "where" maybe-expression [ "map" function-anon ]
    when             ::= "when" maybe-expr newline { { indent } when-case }
    when-case        ::= maybe-expr "then" expr-or-stmt
    
    control-flow-stmt::= loop | while | for | "break" | "continue"
    while            ::= "while" maybe-expr "do" expr-or-stmt
    for              ::= "for" maybe-expr "in" maybe-expr "do" expr-or-stmt
    
    indent           ::= \t | \s\s\s\s
    newline          ::= \n | \r\n

The language uses indentation to denote do-blocks. The indentation amount can't be described in the grammar directly, 
but it does adhere to the following rules:

* Every new expression or statement in a do block must be preceded by n + 1 `indent`'s, where n is the amount of 
  `indent`'s before the do block
* The same holds for every new `when-case` in a `when`

A `maybe-expr` is used in a situation where an expression is required,  but we cannot know in advance whether it will be
an expression or statement without type checking the program.
`expr-or-stmt` may be used when it does not matter whether it is an expression or statement.

A module denotes a single source file, and can be one of the following:
* A `program`: A script, which is to be executed.
    * May contain functions, which are only visible to the script itself.
* A `type`   : An interface which may denote the type of a certain class of object.
    * Contains a collection of definitions, immutable assignements, and functions. All may be viewed as properties of the 
      type. As such, these are all publicly visible.
    * All `util` and `class` that implement `type` must assign to the definitions.
* A `util`   : A collection of functions and immutable assignments.
* A `class`  : A collection of functions that act upon encapsulated data. 
    * Contains a collection of assignments and functions. All assignments and functions are public. 
    * A function or immutable-assignment may be prepended with `util` to make it static.
    * A function or assignment may be prepended with `private` to make it only visible within this module.
               
Note that there is no inheritance. 
