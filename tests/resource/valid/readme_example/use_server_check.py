from ipaddress import IPv4Address
from server import MyServer

class MyServer:
    is_connected: bool = true

    def __init__(self, ip_address: IPv4Address):
        self.ip_address = ip_address

    def last_sent() -> str:
        "dummy"

some_ip = ipaddress.ip_address("151.101.193.140")
my_server = MyServer(some_ip)

http_server.connect()
if my_server.is_connected:
    http_server.send("Hello World!")

# This statement may raise an error, but for now de simply leave it as-is
# See the error handling section for more detail
print(f"last message sent before disconnect: \"{my_server.last_sent()}\".")
my_server.disconnect()
