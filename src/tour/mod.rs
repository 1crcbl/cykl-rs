pub mod array;
pub mod twoleveltree;

pub trait Tour {
    type TourNode: PartialEq + Vertex;

    fn get(&self, node_idx: usize) -> Option<&Self::TourNode>;

    /// Returns the vertex that follows the vertex representing node `node_idx` in the current tour.
    ///
    /// Since a tour is a cycle, this function will return the first vertex,
    /// if `v` is the last vertex of the tour.
    ///
    /// The function returns `None` if the vertex is not found in the data structure
    /// or the tour is empty.
    fn next(&self, node_idx: usize) -> Option<&Self::TourNode>;

    /// Returns the vertex that precedes the vertex representing node `node_idx` in the current tour.
    ///
    /// Since a tour is a cycle, this function will return the last vertex,
    /// if `v` is the first vertex of the tour.
    ///
    /// The function returns `None` if the vertex is not found in the data structure
    /// or the tour is empty.
    fn prev(&self, node_idx: usize) -> Option<&Self::TourNode>;

    /// Returns true iff a tour, starting at the vertex `from_idx`, arrives at the vertex `mid_idx`
    /// before reaching the vertex `to_idx` in its forward traversal.
    fn between(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool;

    /// Updates the tour by replacing the edges `(from_idx1, to_idx1)` and `(from_idx2, to_idx2)`
    /// by the new edges `(from_idx1, from_idx2)` and `(to_idx1, to_idx2)`.
    ///
    /// This function assumes that next(from_idx1) = to_idx1 and next(from_idx2) = to_idx2.
    fn flip(&mut self, from_idx1: usize, to_idx1: usize, from_idx2: usize, to_idx2: usize);
}

pub trait Vertex {
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

    pub fn create_container(n_nodes: usize) -> Container {
        let mut container = Container::new(MetricKind::Euc2d);
        for ii in 0..n_nodes {
            container.add(ii as Scalar, ii as Scalar, ii as Scalar);
        }
        container
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
