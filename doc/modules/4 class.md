# Class

A class describes the properties of an instance of that class. A class encapsulates data and contains definitions. A 
definition is either a value, or an immutable method. A definition may be private.

An instance of a class may be created. A class may not inherit from another class, but may have another class as a 
property, and forward its methods. A class may implement a type. It must then either implement the methods described in
the type, or contain an instance of a class that forwards these methods.

A immutable instantiation may not modify its fields. Therefore, it is not possible to call a method of an instantiation 
that modifies its fields. A method modifies the fields of an instance by assigning to a variable preceded by `self`. 
All fields of a `class` must be either defined in the constructor or at the top of the body of the `class`.

`to_string` and `to_hash` of a class may be either a function or a constant. If these are not implemented, the default
implementation is used.

`eq` may be implemented. If not, the default implementation is used of recursively checking that the values are equal.
For contained instances of the classes, this is only done for forwarded definitions. See "Operator Overloading" for more
details on operator overloading.

The size operator: `| <expression> |` only works if the type of the expression is a `class` which defines `size`.

## Using Type Aliases to Define State of Self

In certain situations, we want to make sure that certain methods can only be called when an instance of a class is in a
certain state. This can be achieved using type aliases and type refinement.

Say I have a type `Server`:

    type Server 
        def connected:          Boolean
        def ip_address:         IPAddress
        def mut last_message:   String
        
        def connect:            (IPAddress) -> Boolean throws [ConnectionErr]
        def last_sent_message:  () -> String
        def send_message:       (String) -> Boolean    throws [ConnectionErr]
        def disconnect:         () -> Boolean
        
    type ServerErr(msg: String) isa Err(msg)

We can do the following:

    type ConnectedHTTPServer isa HTTPServer when
        self connected else ServerErr("Not connected.")
        
    type DiconnectedHTTPServer isa HTTPServer when
        self not connected else ServerErr("Already connected.")
        
    util HTTPServer
        def stateless_message <- "This message is always the same."
        
    class HTTPServer isa Server
        def connected <- false
        def mut last_message <- ""
        
        def init(self: DisconnectedHTTPServer, def ip_address: IPAddress)
        
        def last_sent_message(self): String -> self last_message
        
        def connect (self: DisconnectedHTTPServer, ip_address: IPAddress): Boolean ->
            # perform some operations here
            true
            
        def send_message(mut self: ConnectedHTTPServer, message: String): Boolean ->
            # perform some operations here
            self last_message <- message
            true
            
        def disconnect(self: ConnectedHTTPServer): Boolean ->
            # perform some operations here
            true
