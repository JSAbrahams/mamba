# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    program           ::= do-block
    do-block          ::= { ( statement | expression ) newline }
    expression-or-do  ::= ( expression | newline indent do-block )
    
    statement         ::= "print" expression | assignment | "donothing" | control-flow-stmt | expression "<-" expression
    expression        ::= "(" expression-or-do ")" | "return" expression | arithmetic | control-flow-expr
    
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
    if                ::= ( "if" | "unless" ) expression "then" expression-or-do 
                          [ [ newline ] "else" expression-or-do ]
    when              ::= "when" expression "is" newline { indent when-case }
    when-case         ::= expression "then" expression-or-do
    
    control-flow-expr ::= loop | while | for | "break" | "continue"
    postfix-if        ::= ( statement | expression ) ( "if" | "unless" ) expression
    loop              ::= "loop" expression-or-do
    while             ::= "while" expression "do" expression-or-do
    for               ::= "for" expression "in" expression "do" expression-or-do
    
    indent            ::= \t | \s\s\s\s
    newline           ::= \n | \r\n
