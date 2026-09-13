⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.4 Type Aliases

_Note_ Type aliases and the `type ... when ...` refinement described here are future work.
None of this page is implemented yet.
The traits, classes and error types it builds on are written in current syntax, the `type` declarations are not.

In certain situations, we want to make sure that certain methods can only be called when an instance of a class is in a certain state.
This can be achieved using type aliases and type refinement.

I have a trait `Server`:

    trait Server where
        def is_connected(self) -> Bool
        def last_sent_message(self) -> Str?
        def send_message(self, message: Str) -> Bool ! ServerErr
        def disconnect(self) -> Bool ! ServerErr
    end

    class ServerErr(msg: Str): Exception(msg)

And I define the following type aliases:

    type ConnectedHTTPServer: HTTPServer when
        self.is_connected() else ServerErr("Not connected.")

    type DisconnectedHTTPServer: HTTPServer when
        not self.is_connected() else ServerErr("Already connected.")

We can do the following:

    class HTTPServer(ip_address: IpAddress): Server where
        def mut connected: Bool := False
        def mut last_message: Str? := None

        def is_connected(self) -> Bool := self.connected

        def last_sent_message(self) -> Str? := self.last_message

        def connect(mut self: DisconnectedHTTPServer, ip_address: IpAddress) -> Bool ! ServerErr := do
            # perform some operations here
            self.connected := True
            True
        end

        def send_message(mut self: ConnectedHTTPServer, message: Str) -> Bool ! ServerErr := do
            # perform some operations here
            self.last_message := message
            True
        end

        def disconnect(mut self: ConnectedHTTPServer) -> Bool ! ServerErr := do
            # perform some operations here
            self.connected := False
            True
        end
    end

The state of the server is now part of each method's signature.
`send_message` can only be called on a `ConnectedHTTPServer`, and `connect` only on a `DisconnectedHTTPServer`.
