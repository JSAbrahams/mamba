# Grammar
The grammar of the language in extended Backus-Naur form (EBNF).

    program                 ::= do-block
    statement               ::= "print" expression | identifier
    expression              ::= "(" expression ")"
                             | "return" expression
                             | arithmetic-expression
                             | control-flow-expression
    do-block                ::= ( { ( expresssion | statement ) newline } | newline )
    expression-or-do        ::= ( expression | newline indent do-block )
    
    identifier              ::= assignment | mutable-assignment
    assignment              ::= "let" id "<-" expression
    mutable-assignment      ::= "mutable" assignment
    
    arithmetic-expression   ::= term | unary-operator expression | term additive-operator expression
    term                    ::= factor | factor multiclative-operator expression
    factor                  ::= constant | id
    
    constant                ::= real-constant | integer-constant | boolean-constant | string-constant
    real-constant           ::= digit"."digit
    integer-constant        ::= digit
    boolean-constant        ::= "True" | "False"
    id                      ::= { character }
    string-constant         ::= "\"" { character } "\""
    
    binary-operator         ::= additive-operator 
                             | multiplicative-operator 
                             | equality-operator
                             | relational-operator
                             | binary-logic-operator
    unary-operator          ::= "not" | additive-operator
    
    additive-operator       ::= "+" | "-"
    multiplicative-operator ::= "*" | "/" | "^" | "mod" 
    equality-operator       ::= "equals" | "is" | "notequals" | "isnot"
    relational-operator     ::= "<=" | ">=" | "<" | ">"
    binary-logic-operator   ::= "and" | "or"
                                    
    control-flow            ::= if | when | loop | while | for | "break" | "continue"
    if                      ::= ( "if" | "unless" ) expression "then" expression-or-do 
                                [ [ newline ] "else" expression-or-do ]
    when                    ::= "when" expression newline { indent when-case }
    when-case               ::= expression "then" expression-or-do
    loop                    ::= "loop" expression-or-do
    while                   ::= "while" expression "do" expression-or-do
    for                     ::= "for" expression "in" expression "do" expression-or-do

    indent                  ::= "\t"
    newline                 ::= "\n"
