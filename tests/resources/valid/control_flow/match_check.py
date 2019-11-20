from collections import defaultdict

a = f"d"

print({
    f"b": f"c",
    f"d": f"e",
}[a])

# TODO handle tuples
# def (b, bb, bbb) <- (0, 1, 2)
#
# match (b, bb, bbb)
#    (0, 1, 2) => print "hello world"

nested = f"other"

{
    f"a": f"b",
    f"c": defaultdict(lambda: f"default", {
        f"other": f"even_other",
        f"other_one": f"better_one",
    })[nested],
}[nested]
