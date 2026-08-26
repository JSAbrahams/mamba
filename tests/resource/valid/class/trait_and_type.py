from abc import ABC, abstractmethod

class Named(ABC):
    @abstractmethod
    def name(self) -> str:
        pass

class Person(Named):
    def __init__(self, name: str):
        Named.__init__(self)
        self.name = name
