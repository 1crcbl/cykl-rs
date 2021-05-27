use crate::{Repo, Scalar};

use super::{between, NodeRel, Tour, TourIter, TourNode, TourOrder};

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
pub struct Array {
    repo: Repo,
    nodes: Vec<TourNode>,
    tracker: Vec<usize>,
    total_dist: Scalar,
}

impl Array {
    pub fn new(repo: &Repo) -> Self {
        let nodes: Vec<TourNode> = repo.into_iter().map(|n| TourNode::new(n)).collect();
        let tracker = (0..nodes.len()).collect();

        Self {
            repo: repo.clone(),
            nodes,
            tracker,
            total_dist: 0.,
        }
    }

    #[inline]
    pub(crate) fn swap_at(&mut self, idx_a: usize, idx_b: usize) {
        self.nodes.swap(self.tracker[idx_a], self.tracker[idx_b]);
        self.tracker.swap(idx_a, idx_b);
    }

    // This function is currently in use for testing purposes.
    #[allow(dead_code)]
    pub(crate) fn tracker(&self) -> &Vec<usize> {
        &self.tracker
    }
}

impl Tour for Array {
    fn apply(&mut self, tour: &TourOrder) {
        let tour = tour.order();
        self.total_dist = 0.;

        for ii in 0..tour.len() {
            self.swap_at(tour[ii], self.nodes[ii].data().index());
            self.nodes[ii].visited(false);

            if ii != tour.len() - 1 {
                self.total_dist += self.repo.distance_at(tour[ii], tour[ii + 1]);
            } else {
                self.total_dist += self.repo.distance_at(tour[ii], tour[0]);
            }
        }
    }

    #[inline]
    fn between(&self, from: &TourNode, mid: &TourNode, to: &TourNode) -> bool {
        between(from.index(), mid.index(), to.index())
    }

    #[inline]
    fn between_at(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool {
        between(from_idx, mid_idx, to_idx)
    }

    #[inline]
    fn distance_at(&self, a: usize, b: usize) -> Scalar {
        // TODO: check if nodes belong to the group.
        self.repo
            .distance(&self.get(a).unwrap().data(), &self.get(b).unwrap().data())
    }

    fn flip(&mut self, from_a: &TourNode, to_a: &TourNode, from_b: &TourNode, to_b: &TourNode) {
        self.flip_at(from_a.index(), to_a.index(), from_b.index(), to_b.index())
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
            let n1 = self.nodes[ato_a + ii].index();
            let n2 = self.nodes[afrom_b - ii].index();
            self.swap_at(n1, n2);
        }
    }

    #[inline]
    fn get(&self, node_idx: usize) -> Option<TourNode> {
        match self.nodes.get(self.tracker[node_idx]) {
            Some(node) => Some(TourNode { inner: node.inner }),
            None => None,
        }
    }

    fn relation(&self, _base: &TourNode, _targ: &TourNode) -> NodeRel {
        todo!()
    }

    #[inline]
    fn successor(&self, node: &TourNode) -> Option<TourNode> {
        // TODO: check if a node belongs to this tour/repo.
        self.successor_at(node.index())
    }

    #[inline]
    fn successor_at(&self, node_idx: usize) -> Option<TourNode> {
        if node_idx > self.nodes.len() {
            return None;
        }

        let next_idx = (self.tracker[node_idx] + 1) % self.nodes.len();
        match self.nodes.get(next_idx) {
            Some(node) => Some(TourNode { inner: node.inner }),
            None => None,
        }
    }

    #[inline]
    fn predecessor(&self, node: &TourNode) -> Option<TourNode> {
        // TODO: check if a node belongs to this tour/repo.
        self.predecessor_at(node.index())
    }

    #[inline]
    fn predecessor_at(&self, node_idx: usize) -> Option<TourNode> {
        if node_idx > self.nodes.len() {
            return None;
        }

        let curr_idx = self.tracker[node_idx];
        let prev_idx = if curr_idx == 0 {
            self.nodes.len() - 1
        } else {
            curr_idx - 1
        };

        match self.nodes.get(prev_idx) {
            Some(node) => Some(TourNode { inner: node.inner }),
            None => None,
        }
    }

    #[inline]
    fn reset(&mut self) {
        for vt in &mut self.nodes {
            vt.visited(false);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    fn total_distance(&self) -> Scalar {
        self.total_dist
    }

    fn visited_at(&mut self, kin_index: usize, flag: bool) {
        self.nodes[kin_index].visited(flag);
    }

    fn gen_cands(&mut self, _k: usize) {
        todo!()
    }

    fn itr(&self) -> TourIter {
        TourIter {
            it: self.nodes.iter(),
        }
    }
}
