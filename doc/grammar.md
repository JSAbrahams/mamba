# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    module-import    ::= "from" id ( "use" id [ "as" id ] | "useall" )
    
    module           ::= class | program | type | util
    type             ::= { module-import newline } { newline } "type" id newline 
                         { ( function-def | definition | immutable-asssignment ) newline { newline } }
    util             ::= { module-import newline } { newline } "util" id newline [ "isa" id [ { "," id } ] ]
                         { ( immutable-assignment ) newline { newline } } { ( function-def-bod ) newline { newline } }
    class            ::= { module-import newline } { newline } "class" id newline [ "isa" id [ { "," id } ] ]
                         { ( [ "util" ]  function-def | assignment ) newline { newline } }
    program          ::= { module-import newline } { newline } { function-def newline { newline } } [ do-block ]
    
    function-call    ::= maybe-expr "." id tuple
    function-call-dir::= id tuple
    function-def     ::= "fun" id "(" function-args ")" [ ":" function-type ]
    function-def-bod ::= function-def "->" expr-or-stmt
    function-args    ::= function-type ":" function-type [ "," function-args ]
    function-type    ::= id | static-tuple | function-tuple "->" function-type
    function-tuple   ::= "(" [ function-type { "," function-type } ] ")"
    
    do-block         ::= { { indent } expr-or-stmt newline [ { indent } newline ] }
    
    maybe-expr       ::= expression | tuple | control-flow-expr | reassignment | function-call | function-call-dir 
                      | newline do-block
    tuple            ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
    reassignment     ::= maybe-expr "<-" maybe-expr
    expr-or-stmt     ::= statement | maybe-expr [ ( "if" | "unless" ) maybe_expr ]
                       
    statement        ::= "print" maybe-expr | assignment | "donothing" | control-flow-stmt
    expression       ::= "return" maybe-expr | arithmetic
    
    id               ::= { ( character | number | "_" ) }
    
    assignments      ::= mutable-assign | immutable-assign
    mutable-assign   ::= [ "mutable" ] immutable-assignment
    immutable-assign ::= variable-def "<-" maybe-expr
    definition       ::= "let" id
    
    arithmetic       ::= term | unary maybe-expr | term additive maybe-expr
    term             ::= factor | factor multiclative-operator maybe-expr
    factor           ::= constant | id
    
    constant         ::= number | boolean | string
    number           ::= real | integer | e-notation
    real             ::= digit "." digit
    integer          ::= digit
    e-notation       ::= digit [ "." digit ] ( "e" | "E" ) [ "-" ] digit
    boolean          ::= "True" | "False"
    string           ::= "\"" { character } "\""
    
    unary            ::= "not" | additive
    additive         ::= "+" | "-"
    multiplicative   ::= "*" | "/" | "^" | "mod"  | equality | relational | binary-logic
    equality         ::= "equals" | "is" | "notequals" | "isnot"
    relational       ::= "<=" | ">=" | "<" | ">"
    binary-logic     ::= "and" | "or"
                                     
    control-flow-expr::= if | when
    if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
    when             ::= "when" maybe-expr newline { { indent } when-case }
    when-case        ::= maybe-expr "then" expr-or-stmt
    
    control-flow-stmt::= loop | while | for | "break" | "continue"
    loop             ::= "loop" expr-or-stmt
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
* A `type`   : An interface which may denote the type of a certain class of object
    * Contains a collection of definitions and functions, which all may be viewed as properties of the type. As such,
      these are all publicly visible.
    * Only immutable assignments are possible, not mutable assignments.
    * All `util` and `class` that implement `type` must assign to definitions which have not been assigned to.
* A `util`   : A collection of functions and immutable assignments.
* A `class`  : A collection of functions that act upon encapsulated data. 
    * Contains a collection of assignments and functions. All assignments and functions are public. 
    * A function or assignment may be prepended with `util` to make it private.
               
Note that there is no inheritance. 
