from typing import Option

class MyServer:
    _message: Option[str] = None

    def send(self, message: str):
        self._message := message
