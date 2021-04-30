use crate::Scalar;

pub mod array;
pub mod twoleveltree;

// Note: TourOrder might be elevated to a struct.
pub type TourOrder = Vec<usize>;

pub trait Tour {
    type TourNode: Vertex + PartialEq + std::fmt::Debug;

    fn init(&mut self, tour: Option<&TourOrder>);

    // Returns the number of vertices in the tour.
    fn size(&self) -> usize;

    fn distance(&self, a: &Self::TourNode, b: &Self::TourNode) -> Scalar;

    fn begin(&self) -> Option<&Self::TourNode>;

    fn end(&self) -> Option<&Self::TourNode>;

    fn get(&self, node_idx: usize) -> Option<&Self::TourNode>;

    fn next(&self, node: &Self::TourNode) -> Option<&Self::TourNode>;
    
    /// Returns the vertex that follows the vertex representing node `node_idx` in the current tour.
    ///
    /// Since a tour is a cycle, this function will return the first vertex,
    /// if `v` is the last vertex of the tour.
    ///
    /// The function returns `None` if the vertex is not found in the data structure
    /// or the tour is empty.
    fn next_idx(&self, node_idx: usize) -> Option<&Self::TourNode>;

    fn prev(&self, node: &Self::TourNode) -> Option<&Self::TourNode>;

    /// Returns the vertex that precedes the vertex representing node `node_idx` in the current tour.
    ///
    /// Since a tour is a cycle, this function will return the last vertex,
    /// if `v` is the first vertex of the tour.
    ///
    /// The function returns `None` if the vertex is not found in the data structure
    /// or the tour is empty.
    fn prev_idx(&self, node_idx: usize) -> Option<&Self::TourNode>;

    fn between(&self, from: &Self::TourNode, mid: &Self::TourNode, to: &Self::TourNode) -> bool;

    /// Returns true iff a tour, starting at the vertex `from_idx`, arrives at the vertex `mid_idx`
    /// before reaching the vertex `to_idx` in its forward traversal.
    fn between_idx(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool;

    fn flip(&mut self, from1: &Self::TourNode, to1: &Self::TourNode, from2: &Self::TourNode, to2: &Self::TourNode);

    /// Updates the tour by replacing the edges `(from_idx1, to_idx1)` and `(from_idx2, to_idx2)`
    /// by the new edges `(from_idx1, from_idx2)` and `(to_idx1, to_idx2)`.
    ///
    /// This function assumes that next(from_idx1) = to_idx1 and next(from_idx2) = to_idx2.
    fn flip_idx(&mut self, from_idx1: usize, to_idx1: usize, from_idx2: usize, to_idx2: usize);
}

pub trait Vertex {
    fn index(&self) -> usize;

    fn is_visited(&self) -> bool;

    fn visited(&mut self, flag: bool);
}

fn between(from: usize, mid: usize, to: usize) -> bool {
    if from <= to {
        from <= mid && mid <= to
    } else {
        !(to < mid && mid < from)
    }
}

#[allow(dead_code, unused_imports)]
mod tests {
    use crate::{metric::MetricKind, node::Container, tour::between, Scalar};

    use super::Tour;

    pub fn create_container(n_nodes: usize) -> Container {
        let mut container = Container::new(MetricKind::Euc2d);
        for ii in 0..n_nodes {
            container.add(ii as Scalar, ii as Scalar, ii as Scalar);
        }
        container
    }

    pub fn test_tree_order(tour: &impl Tour, expected: &Vec<usize>) {
        let len = expected.len();
        assert_eq!(tour.get(expected[0]), tour.next_idx(expected[len - 1]));
        assert_eq!(tour.get(expected[len - 1]), tour.prev_idx(expected[0]));

        for ii in 1..(expected.len() - 1) {
            assert_eq!(tour.get(expected[ii]), tour.prev_idx(expected[ii + 1]));
            assert_eq!(tour.get(expected[ii + 1]), tour.next_idx(expected[ii]));
        }

        assert_eq!(tour.get(expected[0]), tour.next(tour.get(expected[len - 1]).unwrap()));
        assert_eq!(tour.get(expected[len - 1]), tour.prev(tour.get(expected[0]).unwrap()));
    }

    #[test]
    fn test_between() {
        // 1 -> 2 -> 3 -> 4 -> 5
        assert!(between(1, 3, 4)); // true
        assert!(!between(1, 5, 4)); // false
        assert!(between(5, 1, 3)); // true
        assert!(!between(5, 3, 1)); // false
    }
}
