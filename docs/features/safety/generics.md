⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.4 ⛑ Safety](README.md)

# 2.4.4 Generics

A `type`, `fuction`, or `class` may have a generic parameter.

To demonstrate generics, we will use an incomplete implementation of a graph.

    type Node
        def id: Int
        def to_hash: Int
        
    class MyNode isa Node:
        def id <- floor(random() * 100)
        def to_hash <- id
        
    class OtherNode isa Node:
        def id <- floor(random() * 1000)
        def to_hash <- id

Next we define a class `Graph`.
```
    from "node" use Node
    
    # a generic is usually a type. It may be a class, but this has limited use as we cannot inherit from a class
    class Graph[N: Node]
        def private mut nodes: Set[N] <- {}
        
        def addNode(node: N) => nodes add node
```

Now we write the main script.
```
    from "graph" use Graph
    from "node" use MyNode, OtherNode
    
    def graph <- Graph[MyNode]
    def other_graph <- Graph[OtherNode]
    
    graph addNode MyNode()
    other_graph addNode OtherNode()
    
    graph addNode OtherNode() # type error! Expected MyNode but got OtherNode
```
