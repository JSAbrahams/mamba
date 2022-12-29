from ipaddress import IPv4Address

class ServerError(Exception):
    def __init__(self, message:str):
        Exception.__init__(self, message)

always_the_same_message: str = "Connected!"

class MyServer:
    is_connected: bool = False
    _last_message: str = "temp"

    def __init__(self, ip_address: IPv4Address):
        self.ip_address = ip_address

    def last_sent(self) -> str:
        return self._last_message

    def connect(self):
        self.is_connected = True
        print(always_the_same_message)

    def send(self, message: str):
        if self.is_connected:
            self._last_message = message
        else:
            raise ServerError("Not connected!")

    def disconnect(self):
        self.is_connected = False
