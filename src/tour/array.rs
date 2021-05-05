use getset::Getters;

use crate::{
    node::{Container, Node},
    Scalar,
};

use super::{between, Tour, TourOrder, Vertex};

//
// Vertex[Tracker[ii]] = n_ii
// Initially:
// Index:    | 0   | 1   | 2   | 3   | 4   | 5   |
// Vertex:   | n_0 | n_1 | n_2 | n_3 | n_4 | n_5 |
// Tracker:  | 0   | 1   | 2   | 3   | 4   | 5   |
//
// After some operations:
// Index:    | 0   | 1   | 2   | 3   | 4   | 5   |
// Vertex:   | n_4 | n_2 | n_3 | n_5 | n_0 | n_1 |
// Tracker:  | 4   | 5   | 1   | 2   | 0   | 3   |
#[derive(Debug)]
pub struct Array<'a> {
    container: &'a Container,
    vertices: Vec<ArrVertex>,
    tracker: Vec<usize>,
    total_dist: Scalar,
}

impl<'a> Array<'a> {
    pub fn new(container: &'a Container) -> Self {
        let vertices: Vec<ArrVertex> = container.into_iter().map(|n| ArrVertex::new(n)).collect();
        let tracker = (0..vertices.len()).collect();

        Self {
            container,
            vertices,
            tracker,
            total_dist: 0.,
        }
    }

    fn swap_at(&mut self, idx_a: usize, idx_b: usize) {
        self.vertices.swap(self.tracker[idx_a], self.tracker[idx_b]);
        self.tracker.swap(idx_a, idx_b);
    }
}

impl<'a> Tour for Array<'a> {
    type TourNode = ArrVertex;

    fn apply(&mut self, tour: &TourOrder) {
        let tour = tour.order();
        self.total_dist = 0.;

        for ii in 0..tour.len() {
            self.swap_at(tour[ii], *&self.vertices[ii].node().index());
            self.vertices[ii].visited = false;

            if ii != tour.len() - 1 {
                self.total_dist += self.container.distance_at(tour[ii], tour[ii + 1]);
            } else {
                self.total_dist += self.container.distance_at(tour[ii], tour[0]);
            }
        }
    }

    #[inline]
    fn between(&self, from: &Self::TourNode, mid: &Self::TourNode, to: &Self::TourNode) -> bool {
        between(from.index(), mid.index(), to.index())
    }

    #[inline]
    fn between_at(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool {
        between(from_idx, mid_idx, to_idx)
    }

    #[inline]
    fn distance_at(&self, a: usize, b: usize) -> Scalar {
        // TODO: check if nodes belong to the group.
        self.container
            .distance(self.get(a).unwrap().node(), self.get(b).unwrap().node())
    }

    fn flip_at(&mut self, from_a: usize, to_a: usize, from_b: usize, to_b: usize) {
        // TODO: this is only a basic implementation.
        // Optimisation on which direction to perform the flip, so that the number of flips
        // is minimised, is not taken into account.
        // (from_a, to_a) - (from_b, to_b) -> (from_a, from_b) - (to_a, to_b)
        if from_a > from_b {
            return self.flip_at(from_b, to_b, from_a, to_a);
        }

        // Converts from node index to internal array index.
        let afrom_b = self.tracker[from_b];
        let ato_a = self.tracker[to_a];
        let diff = (afrom_b - ato_a + 1) / 2;
        for ii in 0..diff {
            let n1 = self.vertices[ato_a + ii].node().index();
            let n2 = self.vertices[afrom_b - ii].node().index();
            self.swap_at(n1, n2);
        }
    }

    #[inline]
    fn get(&self, node_idx: usize) -> Option<&Self::TourNode> {
        self.vertices.get(self.tracker[node_idx])
    }

    #[inline]
    fn next(&self, node: &Self::TourNode) -> Option<&Self::TourNode> {
        // TODO: check if a node belongs to this tour/container.
        self.next_at(node.index())
    }

    #[inline]
    fn next_at(&self, node_idx: usize) -> Option<&Self::TourNode> {
        if node_idx > self.vertices.len() {
            return None;
        }

        let next_idx = (self.tracker[node_idx] + 1) % self.vertices.len();
        self.vertices.get(next_idx)
    }

    #[inline]
    fn prev(&self, node: &Self::TourNode) -> Option<&Self::TourNode> {
        // TODO: check if a node belongs to this tour/container.
        self.prev_at(node.index())
    }

    #[inline]
    fn prev_at(&self, node_idx: usize) -> Option<&Self::TourNode> {
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

    #[inline]
    fn reset(&mut self) {
        for vt in &mut self.vertices {
            vt.visited(false);
        }
    }

    #[inline]
    fn size(&self) -> usize {
        self.vertices.len()
    }

    #[inline]
    fn total_distance(&self) -> Scalar {
        self.total_dist
    }

    fn visited_at(&mut self, kin_index: usize, flag: bool) {
        self.vertices[kin_index].visited(flag);
    }
}

impl<'a> IntoIterator for Array<'a> {
    type Item = ArrVertex;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
    }
}

impl<'a, 's> IntoIterator for &'s Array<'a> {
    type Item = &'s ArrVertex;
    type IntoIter = std::slice::Iter<'s, ArrVertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.iter()
    }
}

#[derive(Debug, Getters, PartialEq)]
pub struct ArrVertex {
    #[getset(get = "pub")]
    node: Node,
    visited: bool,
}

impl ArrVertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
            visited: false,
        }
    }
}

impl Vertex for ArrVertex {
    fn index(&self) -> usize {
        self.node.index()
    }

    fn is_visited(&self) -> bool {
        self.visited
    }

    fn visited(&mut self, flag: bool) {
        self.visited = flag;
    }
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::super::tests::create_container;
    use super::*;

    use crate::{metric::MetricKind, tour::tests::test_tree_order};
    use crate::{node::Container, Scalar};

    #[test]
    fn test_apply() {
        let container = create_container(10);
        let mut tour = Array::new(&container);
        let expected = TourOrder::new(vec![3, 0, 4, 1, 6, 8, 7, 9, 5, 2]);
        tour.apply(&expected);
        test_tree_order(&tour, &expected);
    }

    #[test]
    fn test_total_dist() {
        let container = create_container(4);
        let mut tour = Array::new(&container);
        tour.apply(&TourOrder::new(vec![0, 1, 2, 3]));
        assert_eq!(6. * (2. as Scalar).sqrt(), tour.total_distance());

        tour.apply(&TourOrder::new(vec![1, 3, 0, 2]));
        assert_eq!(8. * (2. as Scalar).sqrt(), tour.total_distance());
    }

    #[test]
    fn test_next() {
        let container = create_container(10);
        let tour = Array::new(&container);

        // [2] -> [3]
        assert_eq!(tour.get(3).unwrap(), tour.next_at(2).unwrap());

        // [4] -> [0]
        assert_eq!(tour.get(0).unwrap(), tour.next_at(9).unwrap());
    }

    #[test]
    fn test_prev() {
        let container = create_container(10);
        let tour = Array::new(&container);

        // [2] -> [3]
        assert_eq!(tour.get(2).unwrap(), tour.prev_at(3).unwrap());

        // [4] -> [0]
        assert_eq!(tour.get(9).unwrap(), tour.prev_at(0).unwrap());
    }

    #[test]
    fn test_swap() {
        let container = create_container(10);
        let mut tour = Array::new(&container);

        // [0] <-> [9]
        tour.swap_at(0, 9);
        test_tree_order(&tour, &TourOrder::new(vec![9, 1, 2, 3, 4, 5, 6, 7, 8, 0]));
    }

    #[test]
    fn test_flip_case_1() {
        let container = create_container(10);
        let mut tour = Array::new(&container);

        tour.flip_at(2, 3, 6, 7);
        let expected = vec![0, 1, 2, 6, 5, 4, 3, 7, 8, 9];
        assert_eq!(expected, tour.tracker);
    }

    #[test]
    fn test_flip_case_2() {
        let container = create_container(10);
        let mut tour = Array::new(&container);

        // Expected: 0 - 1 - 9 - 8 - 7 - 6 - 5 - 4 - 3 - 2
        tour.flip_at(9, 0, 1, 2);
        let expected = vec![0, 1, 9, 8, 7, 6, 5, 4, 3, 2];
        assert_eq!(expected, tour.tracker);
    }

    #[test]
    fn test_between() {
        let container = create_container(10);
        let tour = Array::new(&container);

        // from < to
        assert!(tour.between_at(2, 5, 8));
        assert!(!tour.between_at(2, 1, 8));
        assert!(tour.between_at(2, 2, 8));
        assert!(tour.between_at(2, 8, 8));

        // from > to
        assert!(tour.between_at(8, 1, 2));
        assert!(!tour.between_at(8, 5, 2));
        assert!(tour.between_at(8, 2, 2));
        assert!(tour.between_at(8, 8, 2));

        // from == to
        assert!(tour.between_at(2, 2, 2));
        assert!(!tour.between_at(2, 8, 2));
    }
}
