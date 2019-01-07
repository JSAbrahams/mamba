# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    (* a function definition contains no expressions in the signature *)
    function-call     ::= maybe-expr "." id tuple
    function-def      ::= "fun" id "(" [ { function-arg "," } function-arg ] ")" 
                          [ "->" ( id | static-tuple | function-arg ) ] "is" expr-or-stmt
    function-arg      ::= id ":" ( id | static-tuple | function-arg ) 
    function-sig      ::= static-tuple "->" ( id | static-tuple )
    static-tuple      ::= "(" ( id | static-tuple | function-sig ) { "," ( id | static-tuple | function-sig ) } ")"
    
    program           ::= { ( function-def | do-block ) }
    
    (* a do block is an expression iff last is expression, else statement *)
    do-block          ::= { { indent } expr-or-stmt newline } [ newline ]
    
    (* can be either a statement or expression, must be checked by type checker *)
    maybe-expr        ::= expression | "(" ( maybe-expr [ "," maybe-expr] ")" | control-flow-expr  
                       | maybe-expr "<-" maybe-expr | function-call | newline do-block
    expr-or-stmt      ::= statement | maybe-expr ( [ "<-" maybe_expr ] | ( "if" | "unless" ) maybe_expr )
                       
    statement         ::= "print" maybe-expr | assignment | "donothing" | control-flow-stmt
    expression        ::= "return" maybe-expr | arithmetic
    
    id                ::= { character | number | "_" }
    
    assignment        ::= normal-assignment | mut-assignment
    normal-assignment ::= "let" id "<-" maybe-expr
    mut-assignment    ::= "mutable" assignment
    
    arithmetic        ::= term | unary maybe-expr | term additive maybe-expr
    term              ::= factor | factor multiclative-operator maybe-expr
    factor            ::= constant | id
    
    (* e-notation can either be real or integer. Must be checked by type checker upon use *)
    constant          ::= number | boolean | string
    number            ::= real | integer | e-notation
    real              ::= digit "." digit
    integer           ::= digit
    e-notation        ::= digit [ "." digit ] ( "e" | "E" ) [ "-" ] digit
    boolean           ::= "True" | "False"
    string            ::= "\"" { character } "\""
    
    unary             ::= "not" | additive
    additive          ::= "+" | "-"
    multiplicative    ::= "*" | "/" | "^" | "mod"  | equality | relational | binary-logic
    equality          ::= "equals" | "is" | "notequals" | "isnot"
    relational        ::= "<=" | ">=" | "<" | ">"
    binary-logic      ::= "and" | "or"
                                     
    (* control flow expression may still be statement, should be checked by type checker *)
    control-flow-expr ::= if | when
    if                ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
    when              ::= "when" maybe-expr "is" newline { { indent } when-case }
    when-case         ::= maybe-expr "then" expr-or-stmt
    
    control-flow-stmt ::= loop | while | for | "break" | "continue"
    loop              ::= "loop" expr-or-stmt
    while             ::= "while" maybe-expr "do" expr-or-stmt
    for               ::= "for" maybe-expr "in" maybe-expr "do" expr-or-stmt
    
    indent            ::= \t | \s\s\s\s
    newline           ::= \n | \r\n

The language uses indentation to denote do-blocks. The indentation amount can't be described in the grammar directly, 
but it does adhere to the following rules:

* Every new expression or statement in a do block must be preceded by n + 1 `indent`'s, where n is the amount of 
  `indent`'s before the do block
* The same holds for every new `when-case` in a `when`

A `maybe-expr` is used in a situation where an expression is required,  but we cannot know in advance whether it will be
an expression or statement without type checking the program.
`expr-or-stmt` may be used when it doesn't matter whether it is an expression or statement.