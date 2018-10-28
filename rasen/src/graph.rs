//! Graph building helpers

use std::ops::Index;

use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{algo, Graph as PetGraph, Incoming, Outgoing};

use super::node::*;

/// Convenience wrapper for [`petgraph::Graph`](petgraph::graph::Graph)
#[derive(Debug)]
pub struct Graph {
    graph: PetGraph<Node, u32>,
}

impl Default for Graph {
    /// Create a new empty graph
    fn default() -> Self {
        Self {
            graph: PetGraph::new(),
        }
    }
}

impl Graph {
    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) -> NodeIndex<u32> {
        self.graph.add_node(node)
    }

    /// Add an edge between two nodes in the graph, infering the result type of the origin node
    pub fn add_edge(&mut self, from: NodeIndex<u32>, to: NodeIndex<u32>, index: u32) {
        self.graph.add_edge(from, to, index);
    }

    pub fn has_cycle(&self) -> bool {
        algo::is_cyclic_directed(&self.graph)
    }

    /// List all the outputs of the graph
    #[allow(clippy::needless_lifetimes)]
    pub fn outputs<'a>(&'a self) -> Box<Iterator<Item = NodeIndex<u32>> + 'a> {
        Box::new(self.graph.externals(Outgoing).filter(move |index| {
            match self.graph.node_weight(*index) {
                Some(&Node::Output(_, _, _)) | Some(&Node::Return) => true,
                _ => false,
            }
        }))
    }

    /// List the incoming connections for a node
    #[allow(clippy::needless_lifetimes)]
    pub fn arguments<'a>(
        &'a self,
        index: NodeIndex<u32>,
    ) -> Box<Iterator<Item = NodeIndex<u32>> + 'a> {
        let mut vec: Vec<_> = self.graph.edges_directed(index, Incoming).collect();

        vec.sort_by_key(|e| e.weight());

        Box::new(vec.into_iter().map(|e| e.source()))
    }
}

impl Index<NodeIndex<u32>> for Graph {
    type Output = Node;

    /// Get a node from the graph
    fn index(&self, index: NodeIndex<u32>) -> &Node {
        &self.graph[index]
    }
}
