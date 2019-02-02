# Generics

A `type` or `class` may implement a generic.

To demonstrate generics, we will use an incomplete implementation of a graph.

File `node.mylang`:

    Type Node
        def id: Int
        def to_hash: Int
        
    class MyNode isa Node:
        def id <- floor(random() * 100)
        def to_hash <- id
        
    class OtherNode isa Node:
        def id <- floor(random() * 1000)
        def to_hash <- id

File `graph.mylang`:
   
    from "node" use Node
    
    # a generic is usually a type. It may be a class, but this has limited use as we cannot inherit from a class
    class Graph[N: Node]
        def private mut nodes: Set[N] <- {}
        
        def addNode(node: N) -> nodes add node
        
File `main.mylang`:
    
    from "graph" use Graph
    from "node" use MyNode, OtherNode
    
    def graph <- Graph[MyNode]
    def other_graph <- Graph[OtherNode]
    
    graph addNode MyNode()
    other_graph addNode OtherNode()
    
    graph addNode OtherNode() # type error! Expected MyNode but got OtherNode
    