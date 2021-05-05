use getset::Getters;

use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use crate::{
    node::{Container, Node},
    Scalar,
};

use super::{between, Tour, Vertex};

type RcVertex = Rc<RefCell<TltVertex>>;
type WeakVertex = Weak<RefCell<TltVertex>>;
type RcParent = Rc<RefCell<ParentVertex>>;
type WeakParent = Weak<RefCell<ParentVertex>>;

#[derive(Debug)]
pub struct TwoLevelTree<'a> {
    container: &'a Container,
    vertices: Vec<RcVertex>,
    parents: Vec<RcParent>,
    total_dist: Scalar,
}

impl<'a> TwoLevelTree<'a> {
    pub fn new(container: &'a Container, max_grouplen: usize) -> Self {
        let mut n_parents = container.size() / max_grouplen;
        if container.size() % max_grouplen != 0 {
            n_parents += 1;
        }

        let vertices = container
            .into_iter()
            .map(|n| TltVertex::new(n).to_rc())
            .collect();

        let mut parents = Vec::with_capacity(n_parents);
        parents.push(ParentVertex::new(0, max_grouplen).to_rc());
        for ii in 1..n_parents {
            let p = ParentVertex::new(ii, max_grouplen).to_rc();
            let prev_p = parents.get(ii - 1).unwrap();
            p.borrow_mut().head = prev_p.borrow().tail.clone();

            if ii == n_parents - 1 {
                let first = parents.first().unwrap();
                p.borrow_mut().tail = first.borrow().head.clone();
            }

            parents.push(p);
        }

        Self {
            container,
            vertices,
            parents,
            total_dist: 0.,
        }
    }

    fn reverse_inner_seg(&mut self, from: usize, to: usize) {
        if from > to {
            return self.reverse_inner_seg(to, from);
        }

        let mut diff = to - from + 1;
        diff = diff / 2 + diff % 2;
        for ii in 0..diff {
            self.swap_and_reverse(from + ii, to - ii);
        }
    }

    fn reverse_outer_seg(&mut self, from: usize, to: usize) {
        if from > to {
            return self.reverse_inner_seg(to, from);
        }

        let mut diff = self.parents.len() - to + from + 1;
        diff = diff / 2 +  diff % 2;
        for ii in 0..diff {
            let idx_a = if from >= ii {
                from - ii
            } else {
                self.parents.len() + from - ii
            };

            let idx_b = (ii + to) % self.parents.len();
            self.swap_and_reverse(idx_a, idx_b);
        }
    }

    fn swap_and_reverse(&mut self, parent_index_a: usize, parent_index_b: usize) {
        if parent_index_a == parent_index_b {
            self.parents.get(parent_index_a).unwrap().borrow_mut().reverse();
            return;
        }

        let p_a = self.parents.get(parent_index_a).unwrap();
        let p_b = self.parents.get(parent_index_b).unwrap();

        // exchange rank
        p_a.borrow_mut().rank = parent_index_b;
        p_b.borrow_mut().rank = parent_index_a;

        // exchange head
        let tmp = p_a.borrow().head.clone();
        p_a.borrow_mut().head = p_b.borrow().head.clone();
        p_b.borrow_mut().head = tmp;
        p_a.borrow().head.borrow_mut().right = p_a.borrow().first();
        p_b.borrow().head.borrow_mut().right = p_b.borrow().first();

        // exchange tail
        let tmp = p_a.borrow().tail.clone();
        p_a.borrow_mut().tail = p_b.borrow().tail.clone();
        p_b.borrow_mut().tail = tmp;
        p_a.borrow().tail.borrow_mut().left = p_a.borrow().last();
        p_b.borrow().tail.borrow_mut().left = p_b.borrow().last();

        p_a.borrow_mut().reverse();
        p_b.borrow_mut().reverse();

        self.parents.swap(parent_index_a, parent_index_b);
    }

    // This function is currently in used only for testing purposes.
    #[allow(dead_code)]
    pub (super) fn parent(&self, index: usize) -> RcParent {
        self.parents[index].clone()
    }
}

impl<'a> Tour for TwoLevelTree<'a> {
    type TourNode = TltVertex;

    fn apply(&mut self, tour: &super::TourOrder) {
        let tour = tour.order();

        let p_len = self.parents.len();
        let v_len = self.vertices.len();

        self.total_dist = 0.;

        for ip in 0..p_len {
            let p = self.parents.get(ip).unwrap();

            p.borrow_mut().reset();

            let beg_seg = ip * p.borrow().max_len;
            let end_seg = (beg_seg + p.borrow().max_len).min(v_len);

            for iv in beg_seg..end_seg {
                let v = self.vertices.get(tour[iv]).unwrap();
                v.borrow_mut().rank = iv - beg_seg;
                v.borrow_mut().parent = Rc::downgrade(p);

                if iv == beg_seg {
                    p.borrow_mut().head.borrow_mut().right = Rc::downgrade(v);
                }

                if iv == end_seg - 1 {
                    p.borrow_mut().tail.borrow_mut().left = Rc::downgrade(v);
                }

                p.borrow_mut().children.push_back(v.clone());

                let next_v = self.vertices.get(tour[(iv + 1) % v_len]).unwrap();

                self.total_dist += self
                    .container
                    .distance(&v.borrow().node, &next_v.borrow().node);

                v.borrow_mut().visited = false;
            }
        }
    }

    fn between(&self, from: &Self::TourNode, mid: &Self::TourNode, to: &Self::TourNode) -> bool {
        if let (Some(fp), Some(mp), Some(tp)) = (
            &from.parent.upgrade(),
            &mid.parent.upgrade(),
            &to.parent.upgrade(),
        ) {
            match (Rc::ptr_eq(fp, mp), Rc::ptr_eq(mp, tp), Rc::ptr_eq(tp, fp)) {
                (true, true, true) => {
                    between(from.rank, mid.rank, to.rank) ^ fp.borrow().is_reverse()
                }
                (true, false, false) => fp.borrow().is_reverse() ^ (from.rank <= mid.rank),
                (false, true, false) => mp.borrow().is_reverse() ^ (mid.rank <= to.rank),
                (false, false, true) => tp.borrow().is_reverse() ^ (to.rank <= from.rank),
                (false, false, false) => {
                    between(fp.borrow().rank(), mp.borrow().rank(), tp.borrow().rank())
                }
                // (true, true, false)
                // (true, false, true)
                // (false, true, true)
                _ => panic!("The transitivity requirement is violated."),
            }
        } else {
            false
        }
    }

    fn between_at(&self, from_index: usize, mid_index: usize, to_index: usize) -> bool {
        match (
            self.get(from_index),
            self.get(mid_index),
            self.get(to_index),
        ) {
            (Some(from), Some(mid), Some(to)) => self.between(from, mid, to),
            _ => false,
        }
    }

    fn distance_at(&self, a: usize, b: usize) -> Scalar {
        self.container.distance_at(a, b)
    }

    fn flip_at(&mut self, from_a: usize, to_a: usize, from_b: usize, to_b: usize) {
        if let (Some(from_a), Some(to_a), Some(from_b), Some(to_b)) = (
            self.get(from_a),
            self.get(to_a),
            self.get(from_b),
            self.get(to_b),
        ) {
            // We assume (!) the fact that every node has parent and thus can bypass sanity check.
            let (p_from_a, p_to_a, p_from_b, p_to_b) = (
                from_a.parent.upgrade().unwrap(),
                to_a.parent.upgrade().unwrap(),
                from_b.parent.upgrade().unwrap(),
                to_b.parent.upgrade().unwrap(),
            );

            // Case 1: Either (to_b, from_a) or (to_a, from_b) stays in the same parent node.
            // The order of inputs here is very important for deciding which segment is to be reversed.
            if Rc::ptr_eq(&p_from_a, &p_to_b) && to_b.rank < from_a.rank {
                // TODO: check unwrap()
                return p_from_a
                    .borrow_mut()
                    .reverse_segment(from_a.rank, to_b.rank);
            } else if Rc::ptr_eq(&p_from_b, &p_to_a) && to_a.rank <= from_b.rank {
                return p_from_b
                    .borrow_mut()
                    .reverse_segment(from_b.rank, to_a.rank);
            }

            // Case 2: (from_a, to_a) AND (from_b, to_b) lie in different segments.
            // Since to_a and to_b are direct successors of from_a and from_b, this means that
            // all vertices are at either ends of their corresponding segments. Thus, we only need
            // to reverse their's parent nodes.
            let (pfa_r, pta_r, pfb_r, ptb_r) = (
                p_from_a.borrow().rank(),
                p_to_a.borrow().rank(),
                p_from_b.borrow().rank(),
                p_to_b.borrow().rank(),
            );

            let (diff1, is_inner1) = if pta_r <= pfb_r {
                (pfb_r - pta_r, true)
            } else {
                (self.parents.len() - pta_r + pfb_r, false)
            };

            let (diff2, is_inner2) = if ptb_r <= pfa_r {
                (pfa_r - ptb_r, true)
            } else {
                (self.parents.len() - ptb_r + pfa_r, false)
            };

            let (from, to, is_inner) = if diff1 <= diff2 {
                // Reverses the path (to_a, from_b).
                (pfb_r, pta_r, is_inner1)
            } else {
                // Reverses the path (to_b, from_a).
                (pfa_r, ptb_r, is_inner2)
            };

            if is_inner {
                self.reverse_inner_seg(from, to);
            } else {
                self.reverse_outer_seg(from, to);
            }
        }
    }

    fn get(&self, index: usize) -> Option<&Self::TourNode> {
        match self.vertices.get(index) {
            Some(v) => unsafe { v.as_ref().as_ptr().as_ref() },
            None => None,
        }
    }

    fn next(&self, kin: &Self::TourNode) -> Option<&Self::TourNode> {
        self.next_at(kin.index())
    }

    fn next_at(&self, kin_index: usize) -> Option<&Self::TourNode> {
        if let Some(kin) = self.vertices.get(kin_index) {
            let kin_borrow = kin.borrow();
            match kin_borrow.parent.upgrade() {
                Some(p) => match p.borrow().next(kin_borrow.rank).upgrade() {
                    Some(next) => unsafe { next.as_ref().as_ptr().as_ref() },
                    None => None,
                },
                None => None,
            }
        } else {
            None
        }
    }

    fn prev(&self, kin: &Self::TourNode) -> Option<&Self::TourNode> {
        self.prev_at(kin.index())
    }

    fn prev_at(&self, kin_index: usize) -> Option<&Self::TourNode> {
        if let Some(kin) = self.vertices.get(kin_index) {
            let kin_borrow = kin.borrow();
            match kin_borrow.parent.upgrade() {
                Some(p) => match p.borrow().prev(kin_borrow.rank).upgrade() {
                    Some(prev) => unsafe { prev.as_ref().as_ptr().as_ref() },
                    None => None,
                },
                None => None,
            }
        } else {
            None
        }
    }

    fn reset(&mut self) {
        todo!()
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
        if let Some(kin) = self.vertices.get(kin_index) {
            kin.borrow_mut().visited(flag);
        }
    }
}

#[derive(Debug, Getters)]
pub struct TltVertex {
    /// Sequential ID used inside a parent node to which a vertex belongs.
    ///
    /// If a vertex is not attached to any parent node, `usize::MAX` will be assigned.
    rank: usize,
    /// Reference to a data node that a vertex represents in a tour.
    #[getset(get = "pub")]
    node: Node,
    /// Flag indicating whether a vertex has been visited/processed.
    visited: bool,
    /// Weak reference to a node's parent node.
    parent: WeakParent,
}

impl TltVertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
            rank: usize::MAX,
            visited: false,
            parent: Weak::new(),
        }
    }

    fn to_rc(self) -> RcVertex {
        Rc::new(RefCell::new(self))
    }
}

impl Vertex for TltVertex {
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

impl PartialEq for TltVertex {
    fn eq(&self, other: &Self) -> bool {
        // TODO: expand comparison to pointer.
        self.node == other.node && self.visited == other.visited
    }
}

#[derive(Debug)]
pub (super) struct ParentVertex {
    rank: usize,
    max_len: usize,
    reverse: bool,
    children: VecDeque<RcVertex>,
    offset: usize,
    head: Rc<RefCell<Conduit>>,
    tail: Rc<RefCell<Conduit>>,
}

impl ParentVertex {
    fn new(rank: usize, max_len: usize) -> Self {
        Self {
            rank,
            max_len,
            reverse: false,
            children: VecDeque::with_capacity(max_len),
            offset: 0,
            head: Conduit::new().to_rc(),
            tail: Conduit::new().to_rc(),
        }
    }

    #[inline]
    fn is_reverse(&self) -> bool {
        self.reverse
    }

    fn first(&self) -> WeakVertex {
        let kid = match self.reverse {
            true => self.children.back(),
            false => self.children.front(),
        };

        match kid {
            Some(v) => Rc::downgrade(v),
            None => Weak::new(),
        }
    }

    fn last(&self) -> WeakVertex {
        let kid = match self.reverse {
            true => self.children.front(),
            false => self.children.back(),
        };

        match kid {
            Some(v) => Rc::downgrade(v),
            None => Weak::new(),
        }
    }

    fn next(&self, index: usize) -> WeakVertex {
        let next_index = match self.reverse {
            true => index,
            false => index + 2,
        };

        if next_index == 0 || next_index == self.children.len() + 1 {
            return self.tail.borrow().right.clone();
        } else {
            return match self.children.get(next_index - 1) {
                Some(kin) => Rc::downgrade(kin),
                None => Weak::new(),
            };
        }
    }

    fn prev(&self, index: usize) -> WeakVertex {
        let prev_index = match self.reverse {
            true => index + 2,
            false => index,
        };

        if prev_index == 0 || prev_index == self.children.len() + 1 {
            return self.head.borrow().left.clone();
        } else {
            return match self.children.get(prev_index - 1) {
                Some(kin) => Rc::downgrade(kin),
                None => Weak::new(),
            };
        }
    }

    #[inline]
    fn rank(&self) -> usize {
        self.rank
    }

    #[inline]
    fn reset(&mut self) {
        self.children.clear();
        self.offset = 0;
        self.reverse = false;
    }

    /// Reverses the entire segment and updates the first and last nodes in the conduit layer
    /// to reflect this change in direction.
    #[inline]
    pub (super) fn reverse(&mut self) {
        let tmp = self.head.borrow().right.clone();
        self.head.borrow_mut().right = self.tail.borrow().left.clone();
        self.tail.borrow_mut().left = tmp;
        self.reverse ^= true;
    }

    /// Reverses the segment `(a, b)` in a parent node. The order of the inputs are
    /// not important since the implementation will handle it.
    ///
    /// # Arguments
    /// * a - The rank of a vertex in a parent node at one end of the segment to be reversed.
    /// * b - The rank of a vertex in a parent node at the other end of the segment to be reversed.
    ///
    /// # Panics
    /// Panics if `a` or `b` are out of bounds.
    fn reverse_segment(&mut self, a: usize, b: usize) {
        if a >= self.children.len() || b >= self.children.len() {
            panic!("Attempt to reverse segment: index out of bounds.")
        }

        if a > b {
            return self.reverse_segment(b, a);
        }

        if a == 0 && b == self.children.len() {
            return self.reverse();
        }

        let diff = (b - a + 1) / 2;
        for ii in 0..diff {
            let (v1, v2) = (a + ii, b - ii);
            let tmp_rank = self.children[v1].borrow().rank;
            self.children[v1].borrow_mut().rank = self.children[v2].borrow().rank;
            self.children[v2].borrow_mut().rank = tmp_rank;
            self.children.swap(v1, v2);
        }

        if a == 0 {
            self.head.borrow_mut().right = Rc::downgrade(&self.children.front().unwrap());
        }

        if b == self.children.len() - 1 {
            self.tail.borrow_mut().left = Rc::downgrade(&self.children.back().unwrap());
        }
    }

    #[inline]
    fn to_rc(self) -> RcParent {
        Rc::new(RefCell::new(self))
    }
}

/// Buffer zone between two parent vertices.
#[derive(Debug)]
struct Conduit {
    left: WeakVertex,
    right: WeakVertex,
}

impl Conduit {
    fn new() -> Self {
        Self {
            left: Weak::new(),
            right: Weak::new(),
        }
    }

    fn to_rc(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}
