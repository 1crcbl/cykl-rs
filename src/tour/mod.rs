use getset::Getters;

use crate::Scalar;

mod array;
pub use array::ArrVertex;
pub use array::Array;

mod tlt;
pub use tlt::TltVertex;
pub use tlt::TwoLevelTree;

mod tests;

pub trait Tour {
    type TourNode: Vertex + PartialEq + std::fmt::Debug;

    /// Rearranges the tour's vertices according to the given order.
    // TODO: should return Result<()>.
    fn apply(&mut self, order: &TourOrder);

    /// Returns true iff the tour, starting at the vertex `from`, arrives at the vertex `mid`
    /// before reaching the vertex `to` in its forward traversal.
    fn between(&self, from: &Self::TourNode, mid: &Self::TourNode, to: &Self::TourNode) -> bool;

    /// Returns true iff the tour, starting at the vertex `from_index`, arrives at the vertex `mid_index`
    /// before reaching the vertex `to_index` in its forward traversal.
    fn between_at(&self, from_index: usize, mid_index: usize, to_index: usize) -> bool;

    /// Calculates the distance between two nodes at the given index.
    ///
    /// # Arguments
    /// * a - The index from the container of the tail node in the arc.
    /// * b - The index from the container of the head node in the arc.
    ///
    /// # Panics
    /// Panics if `a` or `b` are out of bounds.
    fn distance_at(&self, a: usize, b: usize) -> Scalar;

    /// Permutate the tour's order by replacing the edges `(from_a, to_a)` and `(from_b, to_b)`
    /// by the new edges `(from_a, from_b)` and `(to_a, to_b)`.
    ///
    /// After flipping, the direction of one of two new edges will be reversed and will be specified
    /// by the concrete implementation of this operation.
    ///
    /// This function assumes `to_a` and `to_b` are the direct successors of `from_b` and `from_b`.
    ///
    /// # Arguments
    /// * from_a - The index from the container of the tail node in the first arc.
    /// * to_a - The index from the container of the head node in the first arc.
    /// * from_b - The index from the container of the tail node in the second arc.
    /// * to_b - The index from the container of the head node in the second arc.
    fn flip_at(&mut self, from_a: usize, to_a: usize, from_b: usize, to_b: usize);

    /// Returns a reference to a vertex representing a node in this tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding vertex, otherwise returns `None`.
    fn get(&self, index: usize) -> Option<&Self::TourNode>;

    /// Returns a reference to a vertex which is the `kin`'s direct successor in the forward
    /// traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding successor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct successor of the last vertex is the first vertex
    /// in the forward traversal of the tour.
    fn next(&self, kin: &Self::TourNode) -> Option<&Self::TourNode>;

    /// Returns a reference to a vertex which is the direct successor of the vertex at the given
    /// index in the forward traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding successor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct successor of the last vertex is the first vertex
    /// in the forward traversal of the tour.
    fn next_at(&self, kin_index: usize) -> Option<&Self::TourNode>;

    /// Returns a reference to a vertex which is the direct predecessor of the vertex at the given
    /// index in the forward traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding predecessor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct predecessor of the last vertex is the first vertex
    /// in the forward traversal of the tour.
    fn prev(&self, kin: &Self::TourNode) -> Option<&Self::TourNode>;

    /// Returns a reference to a vertex which is the direct predecessor of the vertex at the given
    /// index in the forward traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding predecessor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct predecessor of the first vertex is the last vertex
    /// in the forward traversal of the tour.
    fn prev_at(&self, kin_index: usize) -> Option<&Self::TourNode>;

    /// Resets all the internal states of the tour and its vertices.
    fn reset(&mut self);

    /// Returns the number of vertices in the tour.
    fn size(&self) -> usize;

    /// Returns the total distance of completely traversing through the tour.
    fn total_distance(&self) -> Scalar;

    /// Sets the flag `visited` for a vertex at the given index.
    fn visited_at(&mut self, kin_index: usize, flag: bool);
}

pub trait Vertex {
    fn index(&self) -> usize;

    fn is_visited(&self) -> bool;

    fn visited(&mut self, flag: bool);
}

#[derive(Debug, Getters)]
pub struct TourOrder {
    #[getset(get = "pub")]
    order: Vec<usize>,
    #[getset(get = "pub")]
    total_dist: Scalar,
}

impl TourOrder {
    pub fn new(order: Vec<usize>) -> Self {
        Self {
            order: order.clone(),
            total_dist: 0.,
        }
    }

    pub fn with_dist(order: Vec<usize>, total_dist: Scalar) -> Self {
        Self { order, total_dist }
    }
}

fn between(from: usize, mid: usize, to: usize) -> bool {
    if from <= to {
        from <= mid && mid <= to
    } else {
        !(to < mid && mid < from)
    }
}
