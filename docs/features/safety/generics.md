⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.4 ⛑ Safety](README.md)

# 2.4.4 Generics

_Note_ Generics are only partly implemented.
A `class` or `trait` may declare a generic parameter, and a field may be typed with it.
Using that parameter as the type of a method argument, and instantiating a generic class, are both future work.
A function may not declare a generic parameter at all.

A `trait` or `class` may have a generic parameter.

To demonstrate generics, we will use an incomplete implementation of a graph.

    trait Node where
        def id: Int
        def to_hash(self) -> Int
    end

    class MyNode(id: Int): Node where
        def to_hash(self) -> Int := self.id
    end

    class OtherNode(id: Int): Node where
        def to_hash(self) -> Int := self.id * 31
    end

Next we define a class `Graph`.

    from node import Node

    # a generic is usually a trait. It may be a class, but this has limited use as we cannot inherit from a class
    class Graph[N: Node](nodes: Set[N]) where
        def contains(self, node: N) -> Bool := node in self.nodes
    end

Now we write the main script.

    from graph import Graph
    from node import MyNode, OtherNode

    def graph := Graph({ MyNode(1), MyNode(2) })
    def other_graph := Graph({ OtherNode(1) })

    print(graph.contains(MyNode(1)))
    print(other_graph.contains(OtherNode(1)))

    print(graph.contains(OtherNode(1))) # type error! Expected MyNode but got OtherNode
