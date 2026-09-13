⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.1 Script

A script is a series of instructions that are to be executed.
A script is the only type of module that can actually be run.
A script may be accompanied by a set of functions.

Below we have an example script:

    from graph import Graph

    def fin first := 1
    def fin last  := 5

    def nodes := { x | x in first ..= last }
    def graph := Graph(nodes)

    graph.connect(first, 4, 20)
    graph.connect(4, 3, 40)
    graph.connect(3, last, 60)

    def distance := graph.calculate_distance(first, last)

    print("distance travelled from {first} to {last} is {distance}.")

Note that a value is interpolated into a string by wrapping it in `{` and `}`.
