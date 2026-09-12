⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.3 Type

_Note_ This page predates the current syntax.
The `type` keyword shown here is now `trait`, and `<-` is now `:=`.
See the [README](../../../README.md#-types-properties-and-classes) for current syntax.

A type describes the properties a value of that type should have.
Each such property is a definition.
A definition is then either a method or an immutable variable.
Only classes can implement types.

Say I have a server, I could define the type as follows:

    type Server
        def connect:            (mut Self, IPAddress) -> Boolean ! ServerErr
        def last_sent_message:  (Self) -> String
        def send_message:       (mut Self, String) -> Boolean    ! ServerErr
        def disconnect:         (mut Self) -> Boolean

This is akin to an abstract base class in Python, but more compact.
Now any class that implements `Server` must have these definitions.

    class MyServer(def ip_address: IPAddress): Server
        def connected        <- False
        def mut last_message <- None

        def last_sent_message(self): String := self.last_message

        def connect (mut self, ip_address: IPAddress) -> Boolean ! ServerErr :=
            # perform some operations here
            self.connected := true
            True

        def send_message(mut self, message: String) -> Boolean ! ServerErr :=
            # perform some operations here
            self.last_message := message
            True

        def disconnect(mut self) -> Boolean :=
            # perform some operations here
            self.connected := false
            True
