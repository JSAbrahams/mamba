⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.1 Script

A script is a series of instructions that are to be executed.
A script is the only type of module that can actually be run.
A script may be accompanied by a set of functions.

Below we have an example script:
```
    from graph import Graph
    
    def first := 1
    def last  := 5
    def graph := Graph({x | x in N, first <= x <= last})
    
    graph.connect(first, 4, distance := 20)
    graph.connect(4, 3, distance := 40)
    graph.connect(3, last, distance := 60)

    def distance <- Graph.calculate_distance(graph, first, last)
    
    print "distance travelled from [first] to [last] is [distance]."
```
