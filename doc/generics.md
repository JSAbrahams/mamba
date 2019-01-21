# Generics

A `type` or `class` may implement a generic.

To demonstrate generics, we will use an incomplete implementation of a graph.

File `node.mylang`:

    Type Node
        def id
        
    class MyNode isa Node:
        def id <- floor(random() * 100)
        def to_hash <- id

File `graph.mylang`:
   
    from "node" use Node
    
    # A generic is usually a type. It may be a class, but this has limited use as we cannot inherit from a class
    class Graph[N isa Node]
        def mut nodes: Set[N] <- {}
        
        def addNode(node: N) <- nodes add node
        
File `main.mylang`:
    
    from "graph" use Graph
    from "node" use MyNode
    
    def graph <- Graph[MyNode]
    
    graph addNode MyNode()
    