from typing import Optional
j = { 10: 200, 30: 5 }

def f(key: int) -> Optional[int]:
    return j[key] if key in j else None
