x = [1, 2, 3]
y = ["a", "b", "c"]
xy = [[(l, m) for l in x if l > 0] for m in y if m != "c"]
