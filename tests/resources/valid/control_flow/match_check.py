from collections import defaultdict

a = "d"

print({
    "b": "c",
    "d": "e",
}[a])

# TODO handle tuples
# def (b, bb, bbb) <- (0, 1, 2)
#
# match (b, bb, bbb)
#    (0, 1, 2) => print "hello world"

nested = "other"

{
    "a": "b",
    "c": defaultdict(lambda: "default", {
        "other": "even_other",
        "other_one": "better_one",
    })[nested],
}[nested]
