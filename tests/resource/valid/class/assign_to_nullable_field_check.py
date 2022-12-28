from typing import Optional

class MyServer:
    _message: Optional[str] = None

    def send(self, x: str):
        self._message = x
