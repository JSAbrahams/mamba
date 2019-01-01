# Grammar
The grammar of the language in extended Backus-Naur form (EBNF).

    program                     ::= { ( expression | statement ) }
    expression                  ::= "(" expression ")"
                                | "return" expression
                                | identifier
                                | arithmetic-expression
                                | control-flow-expression
    statement                   ::= control-flow-statement | "print" expression
    do-block                    ::= do-block-expression | do-block-statement
    do-block-statement          ::= ( { ( expresssion | statement ) newline } | newline )
    do-block-expression         ::= { ( expresssion | statement ) newline } expression newline
    
    id                          ::= { character }
    identifier                  ::= id
                                | declaration
                                | arithmetic-expression
                                | function-call
    declaration                 ::= pure-declaration 
                                | mutable-declration 
                                | reassignable-declaration
    pure-declaration            ::= id "<-" expression
    mutable-declaration         ::= "mutable" pure-declaration
    reassignable-declaration    ::= "reassignable" ( mutable-declaration | pure-declaration )
    
    constant                    ::= real-constant 
                                | integer-constant 
                                | boolean-constant 
                                | string-constant
    real-constant               ::= digit"."digit
    integer-constant            ::= digit
    boolean-constant            ::= "True" | "False"
    string-constant             ::= { character }
    
    arithmetic-expression       ::= term 
                                | unary-operator term 
                                | term additive-operator term
    term                        ::= factor 
                                | factor multiclative-operator factor
    factor                      ::= constant | expression
    
    binary-operator             ::= additive-operator 
                                | multiplicative-operator 
                                | equality-operator
                                | negational-equality-operator
                                | relational-operator
                                | binary-logic-operator
    unary-operator              ::= "not" | additive-operator
    additive-operator           ::= "+" | "-" | 
    multiplicative-operator     ::= "*" | "/" | "^" | "mod" 
    equality-operator           ::= "equals" | "is"
    negational-equality-operator::= "notequals" | "isnot"
    relational-operator         ::= "<=" | ">=" | "<" | ">"
    binary-logic-operator       ::= "and" | "or" |
                                    
    control-flow-expression     ::= if-expression | when-expression
    if-expression               ::= "if" expression "then" 
                                    ( newline indent do-block-expression | expression ) [ newline ] "else" 
                                    ( newline indent do-block-expression | expression )
    when-expression             ::= "when" expression newline { indent when-case } [ newline indent "else" 
                                    ( newline indent do-block-expression | expression ) ]
    when-case                   ::= "equals" expression "then" ( newline indent do-block-expression | expression )
                                    
    control-flow-statement      ::= ( expression | statement) "if" expression [ "else" 
                                    ( newline indent do-block-expression | expression | statement ) ]
                                | "if" expression "then" ( newline indent do-block | expression | statement ) [ "else"
                                  ( newline indent do-block-expression | expression | statement ) ]
                                | ( expression | statement) "unless" expression
                                | "unless" expression "then" ( newline indent do-block | expression | statement )
                                | "when" expression newline { indent when-case } newline { "else" 
                                  ( newline indent do-block | statement | expression ) }
                                | "while" expression "do" ( newline indent do-block | expression | statement )
                                | "for" expression "in" expression "do" 
                                  ( newline indent do-block | expression | statement )
                                | "loop" (expression | newline indent do-block )
                                | "continue"
                                | "break"

    indent                      ::= \t | \s\s\s\s
    newline                     ::= \n | \r\n
