⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.3 Type

A trait describes the properties a value of that type should have.
Each such property is a definition.
A definition is then either a method or a variable.
Only classes can implement traits.

Say I have a server, I could define the trait as follows:

    trait Server where
        def connect(mut self, ip_address: IpAddress) -> Bool ! ServerErr
        def last_sent_message(self) -> Str?
        def send_message(mut self, message: Str) -> Bool ! ServerErr
        def disconnect(self) -> Bool
    end

This is akin to an abstract base class in Python, but more compact.
A definition in a trait is a signature with no body.
Note that each method takes an explicit `self` argument, and that `self` means the method may not change the state of the instance.

Now any class that implements `Server` must have these definitions.

    class MyServer(ip_address: IpAddress): Server where
        def mut connected: Bool := False
        def mut last_message: Str? := None

        def last_sent_message(self) -> Str? := self.last_message

        def connect(mut self, ip_address: IpAddress) -> Bool ! ServerErr := do
            # perform some operations here
            self.connected := True
            True
        end

        def send_message(mut self, message: Str) -> Bool ! ServerErr := do
            # perform some operations here
            self.last_message := message
            True
        end

        def disconnect(mut self) -> Bool := do
            # perform some operations here
            self.connected := False
            True
        end
    end

The constructor argument `ip_address` is a field, and is reached as `self.ip_address` inside the body.
A class may name more than one parent, as in `class MyServer(...): Server, Named where ... end`.
