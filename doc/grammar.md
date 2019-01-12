# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    module-import    ::= "from" id ( "use" id [ "as" id ] | "useall" )
    util-import      ::= "from" "util" 
    
    module           ::= interface | util | class | script
    interface        ::= { module-import newline } newline { newline } 
                         "type" newline newline { newline } 
                         { ( function-def | function-def-body | definition | immutable-asssign ) newline { newline } } ]
    util             ::= { module-import newline } newline { newline } 
                         "util" id [ "isa" id { "," id } ] newline { newline } 
                         { ( immutable-declaration | function-def-bod ) newline { newline } }
    class            ::= { module-import newline } newline { newline } 
                         "class" id [ "isa" id { "," id } ] newline { newline } 
                         { defer-declaration newline } { newline } 
                         { [ "private" ] ( function-def-bod | declaration ) ) newline { newline } }
    script           ::= { module-import newline } { newline } 
                         { function-def newline { newline } } 
                         [ do-block ]
    
    function-call    ::= [ "self" ] maybe-expr "." id tuple
    function-call-dir::= id tuple
    function-def     ::= "fun" id "(" function-args ")" [ ":" function-type ]
    function-def-bod ::= function-def "->" expr-or-stmt
    function-args    ::= function-type ":" function-type [ "," function-args ]
    function-type    ::= id | static-tuple | function-tuple "->" function-type
    function-tuple   ::= "(" [ function-type { "," function-type } ] ")"
    function-anon    ::= ( id | function-tuple ) "->' maybe-expr
    
    do-block         ::= { { indent } expr-or-stmt newline [ { indent } newline ] }
    
    expr-or-stmt     ::= statement 
                      | maybe-expr [ ( "if" | "unless" ) maybe_expr ]
                      
    statement        ::= "print" maybe-expr 
                      | declaration 
                      | control-flow-stmt
    maybe-expr       ::= "return" [ maybe-expr ] 
                      | operation 
                      | tuple 
                      | function-anon
                      | control-flow-expr 
                      | reassignment 
                      | function-call 
                      | function-call-dir 
                      | newline do-block
    
    id               ::= [ "self" ] { ( character | number | "_" ) }
    tuple            ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
    
    reassignment     ::= maybe-expr "<-" maybe-expr
    defer-declaration::= declaration [ "forward" id { "," id } ]
    declaration      ::= mutable-declaration | immutable-declaration
    mutable-decl     ::= [ "mutable" ] immutable-declaration
    immutable-decl   ::= definition "<-" maybe-expr
    definition       ::= "let" id [ ":" id ]

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
                                     
    control-flow-expr::= if | from | when
    if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
    from             ::= "from" maybe-expr [ newline ] "where" function-anon [ "map" function-anon ]
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
* A `interface` : An interface which may denote the type of a certain class of object.
    * Contains a collection of definitions, immutable assignments, and functions. All may be viewed as properties of the 
      interface. As such, these are all publicly visible.
    * All `util` and `class` that implement `interface` must assign to the definitions.
    * It may contain functions with a body which are the default implementation. This may not be overwritten. As such, 
      these should only be used to implement behaviour that is always the same for the type, as `class`es that actually
      implement the `interface` will not be able to overwrite it.
* A `util`   : A collection of functions and immutable assignments.
* A `class`  : A collection of functions that act upon encapsulated data. 
    * Contains a collection of assignments and functions. All assignments and functions are public.
    * A function or assignment may be prepended with `private` to make it only visible within this module.
    * A class may not inherit from another class as in other OOP languages. Composition is enforced. 
      To avoid having to implement functions in the derived classes if they are only being forwarded, the `forward` 
      keyword may be used. This allows us to pick what behaviour is deferred to the containing object and what behaviour
      is implemented. 
      Inheritance is therefore still indirectly possible, but discouraged, and made more explicit. One can look at the 
      class file and will know exactly what properties it has without having to read any classes it may inherit from.
               