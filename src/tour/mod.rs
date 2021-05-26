use enum_dispatch::enum_dispatch;
use getset::Getters;

use crate::Scalar;

mod array;
pub use array::Array;

mod tll;
pub use tll::TwoLevelList;

mod node;
pub use node::TourNode;

mod tests;

#[enum_dispatch]
// TODO: better name
pub enum TourImpltor {
    Array,
    TwoLevelList,
}

#[enum_dispatch(TourImpltor)]
pub trait Tour {
    /// Rearranges the tour's vertices according to the given order.
    // TODO: should return Result<()>.
    fn apply(&mut self, order: &TourOrder);

    /// Returns true iff the tour, starting at the vertex `from`, arrives at the vertex `mid`
    /// before reaching the vertex `to` in its forward traversal.
    fn between(&self, from: &TourNode, mid: &TourNode, to: &TourNode) -> bool;

    /// Returns true iff the tour, starting at the vertex `from_index`, arrives at the vertex `mid_index`
    /// before reaching the vertex `to_index` in its forward traversal.
    fn between_at(&self, from_index: usize, mid_index: usize, to_index: usize) -> bool;

    fn distance(&self, a: &TourNode, b: &TourNode) -> Scalar {
        self.distance_at(a.index(), b.index())
    }

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

    fn flip(&mut self, from_a: &TourNode, to_a: &TourNode, from_b: &TourNode, to_b: &TourNode);

    /// Returns a reference to a vertex representing a node in this tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding vertex, otherwise returns `None`.
    fn get(&self, index: usize) -> Option<TourNode>;

    /// Returns a reference to a vertex which is the `kin`'s direct successor in the forward
    /// traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding successor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct successor of the last vertex is the first vertex
    /// in the forward traversal of the tour.
    fn successor(&self, kin: &TourNode) -> Option<TourNode>;

    /// Returns a reference to a vertex which is the direct successor of the vertex at the given
    /// index in the forward traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding successor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct successor of the last vertex is the first vertex
    /// in the forward traversal of the tour.
    fn successor_at(&self, kin_index: usize) -> Option<TourNode>;

    /// Returns a reference to a vertex which is the direct predecessor of the vertex at the given
    /// index in the forward traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding predecessor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct predecessor of the last vertex is the first vertex
    /// in the forward traversal of the tour.
    fn predecessor(&self, kin: &TourNode) -> Option<TourNode>;

    /// Returns a reference to a vertex which is the direct predecessor of the vertex at the given
    /// index in the forward traversal of the tour.
    ///
    /// If a node is registered in the container of this tour, returns the reference to its
    /// corresponding predecessor, otherwise returns `None`.
    ///
    /// Since a tour is a cycle, the direct predecessor of the first vertex is the last vertex
    /// in the forward traversal of the tour.
    fn predecessor_at(&self, kin_index: usize) -> Option<TourNode>;

    /// Resets all the internal states of the tour and its vertices.
    fn reset(&mut self);

    /// Returns the number of vertices in the tour.
    fn len(&self) -> usize;

    /// Returns the total distance of completely traversing through the tour.
    fn total_distance(&self) -> Scalar;

    /// Sets the flag `visited` for a vertex at the given index.
    fn visited_at(&mut self, kin_index: usize, flag: bool) {
        match self.get(kin_index) {
            Some(opt) => match opt.inner {
                Some(node) => unsafe {
                    (*node.as_ptr()).visited(flag);
                },
                None => panic!("Missing pointer."),
            },
            None => {}
        }
    }

    /// Generates a set of candidates for all nodes in a tour.
    /// Currently, only the k-nearest-neighbour generator is implemented. This generator can provide
    /// adequate sets as long as all nodes are well distributed. However, if they are tendentially
    /// clustered to each other, triangulation algorithms will deliver a more superior result.
    /// https://en.wikipedia.org/wiki/Fortune%27s_algorithm
    /// Other option is the alpha-nearness. These algorithms will be implemented soon.
    fn gen_cands(&mut self, k: usize);

    fn itr(&self) -> TourIter;
}

pub struct TourIter<'s> {
    it: std::slice::Iter<'s, TourNode>,
}

impl<'s> Iterator for TourIter<'s> {
    type Item = &'s TourNode;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    #[inline]
    #[allow(unused_mut)]
    fn last(mut self) -> Option<Self::Item> {
        self.it.next_back()
    }
}

pub trait STree {
    fn build_mst(&mut self);

    fn cost_m1t(&self) -> HeldKarpBound;
}

pub enum HeldKarpBound {
    Value(Scalar),
    Optimal,
}

#[derive(Debug, Getters)]
pub struct TourOrder {
    #[getset(get = "pub")]
    order: Vec<usize>,
    #[getset(get = "pub")]
    total_dist: Scalar,
}

impl TourOrder {
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            total_dist: 0.,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            order: Vec::with_capacity(capacity),
            total_dist: 0.,
        }
    }

    pub fn with_ord(order: Vec<usize>) -> Self {
        Self {
            order: order,
            total_dist: 0.,
        }
    }

    pub fn with_dist(order: Vec<usize>, total_dist: Scalar) -> Self {
        Self { order, total_dist }
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn add(&mut self, index: usize) {
        self.order.push(index);
    }
}

fn between<T>(from: T, mid: T, to: T) -> bool
where
    T: PartialEq + PartialOrd,
{
    if from <= to {
        from <= mid && mid <= to
    } else {
        !(to < mid && mid < from)
    }
}
