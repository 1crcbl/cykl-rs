use getset::Getters;

use crate::node::Node;

pub mod arraytour;

pub trait Tour {
    /// Returns the vertex that follows the vertex representing node `node_idx` in the current tour.
    ///
    /// Since a tour is a cycle, this function will return the first vertex,
    /// if `v` is the last vertex of the tour.
    ///
    /// The function returns `None` if the vertex is not found in the data structure
    /// or the tour is empty.
    fn next(&self, node_idx: usize) -> Option<&Vertex>;

    /// Returns the vertex that precedes the vertex representing node `node_idx` in the current tour.
    ///
    /// Since a tour is a cycle, this function will return the last vertex,
    /// if `v` is the first vertex of the tour.
    ///
    /// The function returns `None` if the vertex is not found in the data structure
    /// or the tour is empty.
    fn prev(&self, node_idx: usize) -> Option<&Vertex>;

    /// Returns true iff a tour, starting at the vertex `from_idx`, arrives at the vertex `mid_idx`
    /// before reaching the vertex `to_idx` in its forward traversal.
    fn between(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool;

    /// Updates the tour by replacing the edges `(from_idx1, to_idx1)` and `(from_idx2, to_idx2)`
    /// by the new edges `(from_idx1, from_idx2)` and `(to_idx1, to_idx2)`.
    ///
    /// This function assumes that next(from_idx1) = to_idx1 and next(from_idx2) = to_idx2.
    fn flip(&mut self, from_idx1: usize, to_idx1: usize, from_idx2: usize, to_idx2: usize);
}

#[derive(Debug, Getters)]
pub struct Vertex {
    #[getset(get = "pub")]
    node: Node,
}

impl Vertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.node().index() == other.node().index()
    }
}