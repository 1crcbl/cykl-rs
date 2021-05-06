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
type RcSegment = Rc<RefCell<Segment>>;
type WeakSegment = Weak<RefCell<Segment>>;

#[derive(Debug)]
pub struct TwoLevelTree<'a> {
    container: &'a Container,
    vertices: Vec<RcVertex>,
    segments: Vec<RcSegment>,
    total_dist: Scalar,
}

impl<'a> TwoLevelTree<'a> {
    pub fn new(container: &'a Container, max_grouplen: usize) -> Self {
        let mut n_segments = container.size() / max_grouplen;
        if container.size() % max_grouplen != 0 {
            n_segments += 1;
        }

        let vertices = container
            .into_iter()
            .map(|n| TltVertex::new(n).to_rc())
            .collect();

        let mut segments = Vec::with_capacity(n_segments);
        segments.push(Segment::new(0, max_grouplen).to_rc());
        for ii in 1..n_segments {
            let p = Segment::new(ii, max_grouplen).to_rc();
            let prev_p = segments.get(ii - 1).unwrap();
            p.borrow_mut().head = prev_p.borrow().tail.clone();

            if ii == n_segments - 1 {
                let first = segments.first().unwrap();
                p.borrow_mut().tail = first.borrow().head.clone();
            }

            segments.push(p);
        }

        Self {
            container,
            vertices,
            segments,
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

        let mut diff = self.segments.len() - to + from + 1;
        diff = diff / 2 +  diff % 2;
        for ii in 0..diff {
            let idx_a = if from >= ii {
                from - ii
            } else {
                self.segments.len() + from - ii
            };

            let idx_b = (ii + to) % self.segments.len();
            self.swap_and_reverse(idx_a, idx_b);
        }
    }

    fn swap_and_reverse(&mut self, segment_index_a: usize, segment_index_b: usize) {
        if segment_index_a == segment_index_b {
            self.segments.get(segment_index_a).unwrap().borrow_mut().reverse();
            return;
        }

        let p_a = self.segments.get(segment_index_a).unwrap();
        let p_b = self.segments.get(segment_index_b).unwrap();

        // exchange rank
        p_a.borrow_mut().rank = segment_index_b;
        p_b.borrow_mut().rank = segment_index_a;

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

        self.segments.swap(segment_index_a, segment_index_b);
    }

    // This function is currently in used only for testing purposes.
    #[allow(dead_code)]
    pub (super) fn segment(&self, index: usize) -> RcSegment {
        self.segments[index].clone()
    }
}

impl<'a> Tour for TwoLevelTree<'a> {
    type TourNode = TltVertex;

    fn apply(&mut self, tour: &super::TourOrder) {
        let tour = tour.order();

        let p_len = self.segments.len();
        let v_len = self.vertices.len();

        self.total_dist = 0.;

        for ip in 0..p_len {
            let p = self.segments.get(ip).unwrap();

            p.borrow_mut().reset();

            let beg_seg = ip * p.borrow().max_len;
            let end_seg = (beg_seg + p.borrow().max_len).min(v_len);

            for iv in beg_seg..end_seg {
                let v = self.vertices.get(tour[iv]).unwrap();
                v.borrow_mut().rank = iv - beg_seg;
                v.borrow_mut().segment = Rc::downgrade(p);

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
            &from.segment.upgrade(),
            &mid.segment.upgrade(),
            &to.segment.upgrade(),
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
            // We assume (!) the fact that every node is bound to a segment and thus can bypass sanity check.
            let (p_from_a, p_to_a, p_from_b, p_to_b) = (
                from_a.segment.upgrade().unwrap(),
                to_a.segment.upgrade().unwrap(),
                from_b.segment.upgrade().unwrap(),
                to_b.segment.upgrade().unwrap(),
            );

            // Case 1: Either the entire path (to_b, from_a) or (to_a, from_b) resides in the
            // same segment. In this case, we will flip the local path.
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

            // Case 2: (from_a, to_a) AND (from_b, to_b) AND both paths (to_b, from_a) and
            // (to_a, from_b) consist of a sequence of consecutive segments.
            // Since to_a and to_b are direct successors of from_a and from_b, this means that
            // all vertices are either at the head or the tail of their corresponding segments.
            // Thus, we only need to reverse their corresponding segments.
            let (pfa_r, pta_r, pfb_r, ptb_r) = (
                p_from_a.borrow().rank(),
                p_to_a.borrow().rank(),
                p_from_b.borrow().rank(),
                p_to_b.borrow().rank(),
            );

            let (diff1, is_inner1) = if pta_r <= pfb_r {
                (pfb_r - pta_r, true)
            } else {
                (self.segments.len() - pta_r + pfb_r, false)
            };

            let (diff2, is_inner2) = if ptb_r <= pfa_r {
                (pfa_r - ptb_r, true)
            } else {
                (self.segments.len() - ptb_r + pfa_r, false)
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
            match kin_borrow.segment.upgrade() {
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
            match kin_borrow.segment.upgrade() {
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
    /// Sequential ID used inside a segment to which a vertex belongs.
    ///
    /// If a vertex is not attached to any segment, `usize::MAX` will be assigned.
    rank: usize,
    /// Reference to a data node that a vertex represents in a tour.
    #[getset(get = "pub")]
    node: Node,
    /// Flag indicating whether a vertex has been visited/processed.
    visited: bool,
    /// Weak reference to a node's segment.
    segment: WeakSegment,
}

impl TltVertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
            rank: usize::MAX,
            visited: false,
            segment: Weak::new(),
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
pub (super) struct Segment {
    rank: usize,
    max_len: usize,
    reverse: bool,
    children: VecDeque<RcVertex>,
    offset: usize,
    head: Rc<RefCell<Conduit>>,
    tail: Rc<RefCell<Conduit>>,
}

impl Segment {
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

    /// Reverses the path `(a, b)` in a segment. The function assumes that both `a` and `b` lie in
    /// the same segment.
    ///
    /// # Arguments
    /// * a - The rank of a vertex at one end of the segment to be reversed.
    /// * b - The rank of a vertex at the other end of the segment to be reversed.
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
    fn to_rc(self) -> RcSegment {
        Rc::new(RefCell::new(self))
    }
}

/// Buffer zone between two segments.
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
