from typing import Optional
b: Optional[bool] = None
match b:
    case True: 
        print("yes")
    case False: 
        print("no")
    case None: 
        print("unknown")


