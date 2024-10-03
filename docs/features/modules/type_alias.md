⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.4 Type Aliases

In certain situations, we want to make sure that certain methods can only be called when an instance of a class is in a certain state.
This can be achieved using type aliases and type refinement.


I have a type `Server`:
```mamba
    Type Server
        def is_connected:      (mut Self, IPAddress) -> Boolean throws [ServerErr]
        def last_sent_message: (Self) -> String
        def send_message:      (mut Self, String) -> Boolean    throws [ServerErr]
        def is_disconnected:   (mut Self) -> Boolean

    type ServerErr(msg: String): Err(msg)
```

And I define the following type aliases:
```mamba
    type ConnectedHTTPServer: HTTPServer when
        self is_connected else ServerErr("Not connected.")

    type DiconnectedHTTPServer: HTTPServer when
        self not is_connected else ServerErr("Already connected.")
```

We can do the following:
```mamba
    class HTTPServer(mut self: DisconnectedHTTPServer, def ip_address: IPAddress): Server
        def connected        := False
        def mut last_message := None

        def last_sent_message(self): String => self.last_message

        def connect(self: DisconnectedHTTPServer, ip_address: IPAddress) -> Boolean raise [ServerErr] =>
            # perform some operations here
            self.connected := True
            True

        def send_message(self: ConnectedHTTPServer, message: String) -> Boolean raise [ServerErr] =>
            # perform some operations here
            self.last_message := message
            True

        def is_disconnected(self: ConnectedHTTPServer) -> Boolean =>
            # perform some operations here
            self.connected := false
            True
```
