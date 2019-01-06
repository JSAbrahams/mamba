# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    program           ::= do-block
    (* a do block is an expression iff last is expression, else statement *)
    do-block          ::= { { indent } statement-or-expr newline } [ newline ]
    expression-or-do  ::= expression | newline do-block
    expr-or-stmt-or-do::= statement-or-expr | newline do-block
    
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
    if                ::= ( "if" | "unless" ) expression "then" expr-or-stmt-or-do [ "else" expr-or-stmt-or-do ]
    when              ::= "when" expression "is" newline { { indent } when-case }
    when-case         ::= expression "then" expr-or-stmt-or-do
    
    control-flow-stmt ::= loop | while | for | "break" | "continue"
    loop              ::= "loop" expr-or-stmt-or-do
    while             ::= "while" expression "do" expr-or-stmt-or-do
    for               ::= "for" expression "in" expression "do" expr-or-stmt-or-do
    
    indent            ::= \t | \s\s\s\s
    newline           ::= \n | \r\n

The language uses indentation to denote do-blocks. The indentation amount can't be described in the grammar directly, 
but it does adhere to the following rules:

* Every new expression or statement in a do block must be preceded by n + 1 `indent`'s, where n is the amount of 
  `indent`'s before the do block
* The same holds for every new `when-case` in a `when`