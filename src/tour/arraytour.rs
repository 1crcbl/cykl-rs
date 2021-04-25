use getset::Getters;

use crate::node::{Container, Node};

use super::Tour;

///
/// Vertex[Tracker[ii]] = n_ii
/// Initially:
/// Index:    | 0   | 1   | 2   | 3   | 4   | 5   |
/// Vertex:   | n_0 | n_1 | n_2 | n_3 | n_4 | n_5 |
/// Tracker:  | 0   | 1   | 2   | 3   | 4   | 5   |
///
/// After some operations:
/// Index:    | 0   | 1   | 2   | 3   | 4   | 5   |
/// Vertex:   | n_4 | n_2 | n_3 | n_5 | n_0 | n_1 |
/// Tracker:  | 4   | 5   | 1   | 2   | 0   | 3   |
#[derive(Debug)]
pub struct ArrayTour<'a> {
    container: &'a Container,
    vertices: Vec<ArrVertex>,
    tracker: Vec<usize>,
}

impl<'a> ArrayTour<'a> {
    pub fn new(container: &'a Container) -> Self {
        let vertices: Vec<ArrVertex> = container.into_iter().map(|n| ArrVertex::new(n)).collect();
        let tracker = (0..vertices.len()).collect();

        Self {
            container,
            vertices,
            tracker,
        }
    }

    fn swap(&mut self, node_idx1: usize, node_idx2: usize) {
        self.vertices.swap(self.tracker[node_idx1], self.tracker[node_idx2]);
        self.tracker.swap(node_idx1, node_idx2);
    }
}

impl<'a> Tour for ArrayTour<'a> {
    type Output = ArrVertex;

    fn get(&self, node_idx: usize) -> Option<&Self::Output> {
        self.vertices.get(self.tracker[node_idx])
    }

    fn next(&self, node_idx: usize) -> Option<&Self::Output> {
        if node_idx > self.vertices.len() {
            return None;
        }

        let next_idx = (self.tracker[node_idx] + 1) % self.vertices.len();
        self.vertices.get(next_idx)
    }

    fn prev(&self, node_idx: usize) -> Option<&Self::Output> {
        if node_idx > self.vertices.len() {
            return None;
        }

        let curr_idx = self.tracker[node_idx];
        let prev_idx = if curr_idx == 0 {
            self.vertices.len() - 1
        } else {
            curr_idx - 1
        };

        self.vertices.get(prev_idx)
    }

    fn between(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool {
        let mut idx = (self.tracker[from_idx] + 1) % self.vertices.len();
        let target = self.tracker[mid_idx];
        let end_idx = self.tracker[to_idx];

        while idx != end_idx {
            if idx == target {
                return true;
            }
            idx += 1;
        }

        false
    }

    fn flip(&mut self, from_idx1: usize, to_idx1: usize, from_idx2: usize, to_idx2: usize) {
        // TODO: this is only a basic implementation.
        // Optimisation on which directio to take is not taken into account.
        // (from1, to1) - (from2, to2) -> (from1, from2) - (to1, to2)
        if from_idx1 > from_idx2 {
            return self.flip(from_idx2, to_idx2, from_idx1, to_idx1);
        }

        // Converts from node idex to internal array index.
        let afrom_idx2 = self.tracker[from_idx2];
        let ato_idx1 = self.tracker[to_idx1];
        let diff = (afrom_idx2 - ato_idx1 + 1) / 2;
        for ii in 0..diff {
            let n1 = self.vertices[ato_idx1 + ii].node().index();
            let n2 = self.vertices[afrom_idx2 - ii].node().index();
            self.swap(n1, n2);
        }
    }

}

// 
#[derive(Debug, Getters)]
pub struct ArrVertex {
    #[getset(get = "pub")]
    node: Node
}

impl ArrVertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
        }
    }
}

impl PartialEq for ArrVertex {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[allow(dead_code)]
mod tests {
    use super::*;

    use crate::{Scalar, node::Container};
    use crate::metric::MetricKind;

    fn create_container() -> Container {
        let mut container = Container::new(MetricKind::Euc2d);
        for ii in 0..10 {
            container.add(ii as Scalar, ii as Scalar, ii as Scalar);
        }
        container
    }

    #[test]
    fn test_next() {
        let container = create_container();
        let tour = ArrayTour::new(&container);

        // [2] -> [3]
        assert_eq!(tour.get(3).unwrap(), tour.next(2).unwrap());

        // [4] -> [0]
        assert_eq!(tour.get(0).unwrap(), tour.next(9).unwrap());
    }

    #[test]
    fn test_prev() {
        let container = create_container();
        let tour = ArrayTour::new(&container);

        // [2] -> [3]
        assert_eq!(tour.get(2).unwrap(), tour.prev(3).unwrap());

        // [4] -> [0]
        assert_eq!(tour.get(9).unwrap(), tour.prev(0).unwrap());
    }

    #[test]
    fn test_swap() {
        let container = create_container();
        let mut tour = ArrayTour::new(&container);

        // [0] <-> [9]
        tour.swap(0, 9);
        assert_eq!(tour.prev(0).unwrap(), tour.get(8).unwrap());
        assert_eq!(tour.next(8).unwrap(), tour.get(0).unwrap());
        assert_eq!(tour.prev(9).unwrap(), tour.get(0).unwrap());
        assert_eq!(tour.next(0).unwrap(), tour.get(9).unwrap());
        assert_eq!(tour.prev(1).unwrap(), tour.get(9).unwrap());
        assert_eq!(tour.next(9).unwrap(), tour.get(1).unwrap());
    }

    #[test]
    fn test_flip_case_1() {
        let container = create_container();
        let mut tour = ArrayTour::new(&container);

        tour.flip(2, 3, 6, 7);
        let expected = vec![0, 1, 2, 6, 5, 4, 3, 7, 8, 9];
        assert_eq!(expected, tour.tracker);
    }

    #[test]
    fn test_flip_case_2() {
        let container = create_container();
        let mut tour = ArrayTour::new(&container);

        // Expected: 0 - 1 - 9 - 8 - 7 - 6 - 5 - 4 - 3 - 2
        tour.flip(9, 0, 1, 2);
        let expected = vec![0, 1, 9, 8, 7, 6, 5, 4, 3, 2];
        assert_eq!(expected, tour.tracker);
    }

    #[test]
    fn test_between() {
        let container = create_container();
        let tour = ArrayTour::new(&container);

        assert!(tour.between(2, 5, 8));
        assert!(!tour.between(2, 1, 8));
        assert!(tour.between(8, 1, 2));
        assert!(!tour.between(8, 5, 2));
    }
}