from ipaddress import IPv4Address
from typing import Option

class ServerError(Exception)
    def __init__(self, message:str):
        Exception.__init__(message)

always_the_same_message = "Connected!"

class MyServer(def ip_address: IPv4Address)
    def is_connected: bool = False
    def _last_message: Option[str] = None

    def __init__(ip_address: IPv4Address):
        self.ip_address = ip_address

    def last_sent(self) -> str:
        if self._last_message != None:
            return self._last_message
        else:
            raise ServerError("No last message!")

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
