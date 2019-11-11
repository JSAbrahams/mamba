from collections import defaultdict

a = "b"

{
    "b" : c,
    "d" : e
}[a]

(b, bb, bbb) = (0, 1, 2)

{
    (0, 1, 2) : print(f"hello world")
}[(b, bb, bbb)]

nested = "other"

{
    "a" : "b",
    "c" : defaultdict(lambda : default, {
        "other" : "even_other",
        "other_one" : "better_one"
    })[c]
}[nested]
