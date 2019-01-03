# Grammar
The grammar of the language in Extended Backus-Naur Form (EBNF).

    program               ::= do-block
    statement             ::= "print" expression | identifier
    expression            ::= "(" expression ")" | "return" expression | arithmetic | control-flow
    do-block              ::= { ( expresssion | statement ) newline }
    expression-or-do      ::= ( expression | newline indent do-block )
    
    id                    ::= { character }
    identifier            ::= assignment | mutable-assignment
    assignment            ::= "let" id "<-" expression
    mutable-assignment    ::= "mutable" assignment
    
    arithmetic            ::= term | unary-operator expression | term additive-operator expression
    term                  ::= factor | factor multiclative-operator expression
    factor                ::= constant | id
    
    (* e-notation can either be real or integer. Must be checked by type checker upon use *)
    constant              ::= number | boolean | string
    number                ::= real | integer | e-notation
    real                  ::= digit "." digit
    integer               ::= digit
    e-notation            ::= digit [ "." digit ] "e" [ "-" ] digit
    boolean               ::= "True" | "False"
    string                ::= "\"" { character } "\""
    
    binary-operator       ::= additive | multiplicative | equality | relational | binary-logic
    unary                 ::= "not" | additive-operator
    
    additive              ::= "+" | "-"
    multiplicative        ::= "*" | "/" | "^" | "mod" 
    equality              ::= "equals" | "is" | "notequals" | "isnot"
    relational            ::= "<=" | ">=" | "<" | ">"
    binary-logic          ::= "and" | "or"
                                    
    control-flow          ::= if | when | loop | while | for | "break" | "continue"
    if                    ::= ( "if" | "unless" ) expression "then" expression-or-do 
                                [ [ newline ] "else" expression-or-do ]
    when                  ::= "when" expression newline { indent when-case }
    when-case             ::= expression "then" expression-or-do
    loop                  ::= "loop" expression-or-do
    while                 ::= "while" expression "do" expression-or-do
    for                   ::= "for" expression "in" expression "do" expression-or-do
    
    indent                ::= \t
    newline               ::= \n
