def greet(s: str) -> str:
    match s:
        case "hello":
            return "hi"
        case "bye":
            return "cya"
        case _:
            return "?"

print(greet("hello"))
print(greet("bye"))
print(greet("what"))

