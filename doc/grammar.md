# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    program           ::= do-block
    do-block          ::= { statement-or-expr newline } [ newline ]
    expression-or-do  ::= ( expression | newline indent do-block ) 
    
    (* assignment is a statement *)
    statement-or-expr ::= ( statement | expression ) | expression "<-" expression-or-do | postfix-if
    statement         ::= "print" expression | assignment | "donothing" | control-flow-stmt
    expression        ::= "(" expression-or-do ")" | "return" expression | arithmetic | control-flow-expr
    postfix-if        ::= ( statement-or-expr ) ( "if" | "unless" ) expression-or-do
    
    id                ::= { character }
    assignment        ::= normal-assignment | mut-assignment
    normal-assignment ::= "let" id "<-" expression
    mut-assignment    ::= "mutable" assignment
    
    arithmetic        ::= term | unary expression | term additive expression
    term              ::= factor | factor multiclative-operator expression
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
    if                ::= ( "if" | "unless" ) expression "then" expression-or-do [ "else" expression-or-do ]
    when              ::= "when" expression "is" newline { indent when-case }
    when-case         ::= expression "then" expression-or-do
    
    control-flow-stmt ::= loop | while | for | "break" | "continue"
    loop              ::= "loop" expression-or-do
    while             ::= "while" expression "do" expression-or-do
    for               ::= "for" expression "in" expression "do" expression-or-do
    
    indent            ::= \t | \s\s\s\s
    newline           ::= \n | \r\n
