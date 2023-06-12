use egui::Pos2;
use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction::{self, Incoming, Outgoing},
    EdgeType,
};

use crate::{
    metadata::Metadata,
    state_computed::{StateComputed, StateComputedNode},
    Edge, Node,
};

/// Encapsulates graph access and traversal methods.
pub struct GraphWrapper<'a, N: Clone, E: Clone, Ty: EdgeType> {
    g: &'a mut StableGraph<Node<N>, Edge<E>, Ty>,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> GraphWrapper<'a, N, E, Ty> {
    pub fn new(g: &'a mut StableGraph<Node<N>, Edge<E>, Ty>) -> Self {
        Self { g }
    }

    pub fn walk(
        &self,
        mut walker_node: impl FnMut(&Self, &NodeIndex, &Node<N>),
        mut walker_edge: impl FnMut(&Self, &EdgeIndex, &Edge<E>),
    ) {
        self.nodes().for_each(|(idx, n)| walker_node(self, &idx, n));
        self.edges().for_each(|(idx, e)| walker_edge(self, &idx, e));
    }

    pub fn node_by_pos(
        &self,
        comp: &'a StateComputed,
        meta: &'a Metadata,
        pos: Pos2,
    ) -> Option<(NodeIndex, &Node<N>, &StateComputedNode)> {
        // transform pos to graph coordinates
        let pos_in_graph = (pos - meta.pan).to_vec2() / meta.zoom;
        self.nodes_with_context(comp)
            .find(|(_, n, comp)| (n.location() - pos_in_graph).length() <= comp.radius(meta))
    }

    pub fn nodes_with_context(
        &'a self,
        comp: &'a StateComputed,
    ) -> impl Iterator<Item = (NodeIndex, &Node<N>, &StateComputedNode)> {
        self.g
            .node_references()
            .map(|(i, n)| (i, n, comp.node_state(&i).unwrap()))
    }

    pub fn nodes(&'a self) -> impl Iterator<Item = (NodeIndex, &Node<N>)> {
        self.g.node_references()
    }

    pub fn edges(&'a self) -> impl Iterator<Item = (EdgeIndex, &Edge<E>)> {
        self.g.edge_references().map(|e| (e.id(), e.weight()))
    }

    pub fn node(&self, i: NodeIndex) -> Option<&Node<N>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex) -> Option<&Edge<E>> {
        self.g.edge_weight(i)
    }

    pub fn edge_endpoints(&self, i: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        self.g.edge_endpoints(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex) -> Option<&mut Node<N>> {
        self.g.node_weight_mut(i)
    }

    pub fn is_directed(&self) -> bool {
        self.g.is_directed()
    }

    pub fn edges_num(&self, idx: NodeIndex) -> usize {
        if self.is_directed() {
            self.g
                .edges_directed(idx, Outgoing)
                .chain(self.g.edges_directed(idx, Incoming))
                .count()
        } else {
            self.g.edges(idx).count()
        }
    }

    pub fn edges_directed(
        &self,
        idx: NodeIndex,
        dir: Direction,
    ) -> impl Iterator<Item = EdgeReference<Edge<E>>> {
        self.g.edges_directed(idx, dir)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use egui::Vec2;

    fn create_test_graph() -> StableGraph<Node<()>, Edge<()>> {
        let mut graph = StableGraph::<Node<()>, Edge<()>>::new();
        let a = graph.add_node(Node::new(Vec2::default(), ()));
        let b = graph.add_node(Node::new(Vec2::default(), ()));
        let c = graph.add_node(Node::new(Vec2::default(), ()));
        let d = graph.add_node(Node::new(Vec2::default(), ()));

        graph.add_edge(a, b, Edge::new(()));
        graph.add_edge(b, c, Edge::new(()));
        graph.add_edge(c, d, Edge::new(()));
        graph.add_edge(a, d, Edge::new(()));

        graph
    }

    #[test]
    fn test_walk() {
        let mut graph = create_test_graph();
        let graph_wrapped = GraphWrapper::new(&mut graph);
        let mutable_string = RefCell::new(String::new());

        graph_wrapped.walk(
            |g, idx, n| {
                assert_eq!(g.node(*idx), Some(n));

                mutable_string.borrow_mut().push('n');
            },
            |g, idx, e| {
                assert_eq!(g.edge(*idx), Some(e));

                mutable_string.borrow_mut().push('e');
            },
        );

        assert_eq!(mutable_string.into_inner(), "nnnneeee".to_string());
    }
}
