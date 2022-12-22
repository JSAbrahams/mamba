from ipaddress import IPv4Address
from server import MyServer

class MyServer:
    def __init__(self, ip_address: IPv4Address):
        self.ip_address = ip_address

fin some_ip = ipaddress.ip_address("151.101.193.140")
my_server = MyServer(some_ip)

def message = "Hello World!"
try:
    my_server.send(message)
except ServerErr as err:
    print(f"Error while sending message: \"{message}\": {err}")

if isinstance(my_server, ConnectedMyServer):
    my_server.disconnect()
