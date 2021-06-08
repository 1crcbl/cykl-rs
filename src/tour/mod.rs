use enum_dispatch::enum_dispatch;
use getset::{CopyGetters, Getters};

use crate::Scalar;

mod array;
pub use array::Array;

mod tll;
pub use tll::TwoLevelList;

mod node;
pub use node::NodeStatus;
pub use node::TourNode;

mod error;
pub use error::UpdateTourError;

pub mod tests;

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
    fn apply(&mut self, order: &TourOrder) -> Result<(), UpdateTourError>;

    /// Returns true iff the tour, starting at the vertex `from`, arrives at the vertex `mid`
    /// before reaching the vertex `to` in its forward traversal.
    fn between(&self, from: &TourNode, mid: &TourNode, to: &TourNode) -> bool;

    /// Returns true iff the tour, starting at the vertex `from_index`, arrives at the vertex `mid_index`
    /// before reaching the vertex `to_index` in its forward traversal.
    fn between_at(&self, from_index: usize, mid_index: usize, to_index: usize) -> bool;

    #[inline]
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

    /// Returns the relation between two nodes.
    ///
    /// If ```base``` precedes ```targ```, [`NodeRel::Predecessor`] is returned.
    /// And if ```base``` is ```targ```'s successor, [`NodeRel::Successor`] is returned instead.
    /// Otherwise, the two nodes are not neighbours which results in [`NodeRel::None`].
    fn relation(&self, base: &TourNode, targ: &TourNode) -> NodeRel;

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

    /// Reverses a tour entirely.
    fn rev(&mut self);

    /// Returns the node order of a tour.
    fn tour_order(&self) -> TourOrder;

    /// Resets all the internal states of the tour and its vertices.
    fn reset(&mut self);

    /// Returns the number of vertices in the tour.
    fn len(&self) -> usize;

    /// Returns the total distance of completely traversing through the tour.
    fn total_distance(&self) -> Scalar;

    /// Returns the iterator over all nodes stored in a tour.
    fn itr(&self) -> TourIter;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeRel {
    Predecessor,
    Successor,
    None,
}

pub struct TourIter<'s> {
    it: std::slice::Iter<'s, TourNode>,
}

impl<'s> Iterator for TourIter<'s> {
    type Item = TourNode;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|n| TourNode { inner: n.inner })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    #[inline]
    #[allow(unused_mut)]
    fn last(mut self) -> Option<Self::Item> {
        // self.it.next_back()
        todo!()
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

#[derive(Debug, CopyGetters, Getters)]
pub struct TourOrder {
    #[getset(get = "pub")]
    order: Vec<usize>,
    #[getset(get_copy = "pub")]
    cost: Scalar,
}

impl TourOrder {
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            cost: Scalar::MAX,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            order: Vec::with_capacity(capacity),
            cost: 0.,
        }
    }

    pub fn with_nat_ord(n: usize) -> Self {
        Self {
            order: (0..n).collect(),
            cost: 0.,
        }
    }

    pub fn with_ord(order: Vec<usize>) -> Self {
        Self {
            order: order,
            cost: 0.,
        }
    }

    pub fn with_cost(order: Vec<usize>, cost: Scalar) -> Self {
        Self { order, cost }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.order.len()
    }

    #[inline]
    pub fn add(&mut self, index: usize) {
        self.order.push(index);
    }
}

impl Default for TourOrder {
    fn default() -> Self {
        Self {
            order: Vec::with_capacity(0),
            cost: Scalar::MAX,
        }
    }
}

/// Combines multiple ```Range``` into a vector.
#[macro_export]
macro_rules! combine_range {
    ($x:expr) => {
        ($x).collect::<Vec<usize>>()
    };
    ($x:expr, $($y:expr),+) => {{
        let mut a: Vec<usize> = ($x).collect();
        a.append(&mut combine_range!($($y),*));
        a
    }}
}

/// Creates an instance of [`TourOrder`] from a list of [`Range`]s.
#[macro_export]
macro_rules! tour_order {
    ($($x:expr),+) => {
        TourOrder::with_ord(combine_range!($($x),+))
    };
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
