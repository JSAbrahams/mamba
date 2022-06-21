from abc import ABC, abstractmethod

class MyType(ABC):
    @abstractmethod
    def abstract_fun(my_arg: int) -> str:
        pass

    def concrete_fun(x: int) -> int:
        return x + 10
