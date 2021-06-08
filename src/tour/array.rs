use crate::{Repo, Scalar};

use crate::tour::{
    between, NodeRel, NodeStatus, Tour, TourIter, TourNode, TourOrder, UpdateTourError,
};

#[derive(Debug)]
pub struct Array {
    repo: Repo,
    nodes: Vec<TourNode>,
    tracker: Vec<usize>,
    total_dist: Scalar,
    rev: bool,
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
            rev: false,
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
    fn apply(&mut self, tour: &TourOrder) -> Result<(), UpdateTourError> {
        let order = tour.order();
        self.total_dist = 0.;
        self.rev = false;

        if order.len() != self.len() {
            Err(UpdateTourError::TourLenMismatched {
                expected: self.len(),
                received: order.len(),
            })?
        }

        for ii in 0..order.len() {
            self.swap_at(order[ii], self.nodes[ii].data().index());
            self.nodes[ii].set_status(NodeStatus::Active);

            if ii != order.len() - 1 {
                self.total_dist += self.repo.distance_at(order[ii], order[ii + 1]);
            } else {
                self.total_dist += self.repo.distance_at(order[ii], order[0]);
            }
        }

        Ok(())
    }

    #[inline]
    fn between(&self, from: &TourNode, mid: &TourNode, to: &TourNode) -> bool {
        between(from.index(), mid.index(), to.index())
    }

    #[inline]
    fn between_at(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool {
        match (
            self.tracker.get(from_idx),
            self.tracker.get(mid_idx),
            self.tracker.get(to_idx),
        ) {
            (Some(f), Some(m), Some(t)) => between(*f, *m, *t),
            _ => false,
        }
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
        let len = self.tracker.len();

        let (mut tfa, mut tfb) = if self.rev {
            (self.tracker[to_a], self.tracker[to_b])
        } else {
            (self.tracker[from_a], self.tracker[from_b])
        };

        if tfb < tfa {
            std::mem::swap(&mut tfa, &mut tfb);
        }

        let tta = (tfa + 1) % len;
        let ttb = (tfb + 1) % len;

        let d1 = tfb - tta + 1;
        let d2 = len - d1;

        if d1 <= d2 {
            let d1 = d1 / 2;
            for ii in 0..d1 {
                let n1 = self.nodes[tta + ii].index();
                let n2 = self.nodes[tfb - ii].index();
                self.swap_at(n1, n2);
            }
        } else {
            let d2 = d2 / 2;
            for ii in 0..d2 {
                let n1 = self.nodes[(len + tfa - ii) % len].index();
                let n2 = self.nodes[(ttb + ii) % len].index();
                self.swap_at(n1, n2);
            }
        }
    }

    #[inline]
    fn get(&self, node_idx: usize) -> Option<TourNode> {
        match self.nodes.get(self.tracker[node_idx]) {
            Some(node) => Some(TourNode { inner: node.inner }),
            None => None,
        }
    }

    fn relation(&self, base: &TourNode, targ: &TourNode) -> NodeRel {
        match (self.successor(base), self.predecessor(base)) {
            (Some(s), Some(p)) => {
                if s.inner == targ.inner {
                    NodeRel::Predecessor
                } else if p.inner == targ.inner {
                    NodeRel::Successor
                } else {
                    NodeRel::None
                }
            }
            _ => NodeRel::None,
        }
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

        let len = self.nodes.len();
        let curr_idx = self.tracker[node_idx];
        let next_idx = if self.rev {
            (len + curr_idx - 1) % len
        } else {
            (curr_idx + 1) % len
        };

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

        let len = self.nodes.len();
        let curr_idx = self.tracker[node_idx];
        let prev_idx = if self.rev {
            (curr_idx + 1) % len
        } else {
            (len + curr_idx - 1) % len
        };
        // let prev_idx = if curr_idx == 0 {
        //     self.nodes.len() - 1
        // } else {
        //     curr_idx - 1
        // };

        match self.nodes.get(prev_idx) {
            Some(node) => Some(TourNode { inner: node.inner }),
            None => None,
        }
    }

    #[inline]
    fn rev(&mut self) {
        self.rev ^= true;
    }

    fn tour_order(&self) -> TourOrder {
        if self.nodes.len() == 0 {
            return TourOrder::default();
        }

        let mut result = Vec::with_capacity(self.nodes.len());
        let mut d = 0.;

        let mut it = self.nodes.iter().peekable();
        while let Some(node) = it.next() {
            result.push(node.index());

            if let Some(next) = it.peek() {
                d += self.distance(node, *next);
            }
        }

        if let (Some(first), Some(last)) = (self.nodes.first(), self.nodes.last()) {
            d += self.distance(last, first);
        }

        TourOrder::with_cost(result, d)
    }

    #[inline]
    fn reset(&mut self) {
        for vt in &mut self.nodes {
            vt.set_status(NodeStatus::Active);
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

    fn itr(&self) -> TourIter {
        TourIter {
            it: self.nodes.iter(),
        }
    }
}
