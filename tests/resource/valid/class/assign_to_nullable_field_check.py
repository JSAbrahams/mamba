from typing import Optional

class MyServer:
    _message: Optional[str] = None

    def send(self, message: str):
        self._message = message
