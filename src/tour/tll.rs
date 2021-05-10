use std::ptr::NonNull;

use crate::{
    node::{Container, Node},
    Scalar,
};

use super::{between, Tour, TourOrder, Vertex};

#[derive(Debug)]
pub struct TwoLevelList<'a> {
    container: &'a Container,
    segments: Vec<Option<NonNull<Segment>>>,
    vertices: Vec<Option<NonNull<TllNode>>>,
    total_dist: Scalar,
}

impl<'a> TwoLevelList<'a> {
    pub fn new(container: &'a Container, max_grouplen: usize) -> Self {
        let mut n_segments = container.size() / max_grouplen;
        if container.size() % max_grouplen != 0 {
            n_segments += 1;
        }

        let mut segments = Vec::with_capacity(n_segments);
        segments.push(to_nonnull(Segment::new(0, max_grouplen)));

        for ii in 1..n_segments {
            let s = to_nonnull(Segment::new(ii, max_grouplen));
            match segments.last() {
                Some(el) => match el {
                    Some(last) => unsafe {
                        (*s.unwrap().as_ptr()).prev = *el;
                        (*last.as_ptr()).next = s;
                    },
                    None => {}
                },
                None => {}
            }

            if ii == n_segments - 1 {
                match segments.first() {
                    Some(el) => match el {
                        Some(first) => unsafe {
                            (*s.unwrap().as_ptr()).next = *el;
                            (*first.as_ptr()).prev = s;
                        },
                        None => {}
                    },
                    None => {}
                }
            }

            segments.push(s);
        }

        let vertices = container
            .into_iter()
            .map(|node| to_nonnull(TllNode::new(node)))
            .collect();

        Self {
            container,
            vertices: vertices,
            segments: segments,
            total_dist: 0.,
        }
    }

    pub fn with_default_order(container: &'a Container, max_grouplen: usize) -> Self {
        let mut result = Self::new(container, max_grouplen);
        result.apply(&TourOrder::new((0..container.size()).collect()));
        result
    }

    fn get_nn(&self, index: usize) -> Option<&Option<NonNull<TllNode>>> {
        self.vertices.get(index)
    }

    #[allow(dead_code)]
    pub(super) fn segment(&self, index: usize) -> Option<&Option<NonNull<Segment>>> {
        self.segments.get(index)
    }
}

impl<'a> Tour for TwoLevelList<'a> {
    type TourNode = TllNode;

    fn apply(&mut self, tour: &super::TourOrder) {
        let order = tour.order();
        let v_len = self.vertices.len();

        self.total_dist = 0.;
        for (sidx, els) in self.segments.iter().enumerate() {
            match els {
                Some(seg) => unsafe {
                    (*seg.as_ptr()).reset();

                    let max_len = seg.as_ref().max_len;
                    let beg_seg = sidx * max_len;
                    let end_seg = (beg_seg + max_len).min(v_len);

                    for iv in beg_seg..end_seg {
                        let el_v = self.vertices.get(order[iv]).unwrap();
                        let el_next = self.vertices.get(order[(iv + 1) % v_len]).unwrap();
                        let el_prev = self.vertices.get(order[(v_len + iv - 1) % v_len]).unwrap();

                        match el_v {
                            Some(vtx) => {
                                (*vtx.as_ptr()).predecessor = *el_prev;
                                (*vtx.as_ptr()).successor = *el_next;
                                (*vtx.as_ptr()).rank = (iv - beg_seg) as i32;
                                (*vtx.as_ptr()).segment = *els;
                            }
                            None => {}
                        }

                        match el_next {
                            Some(vtx) => {
                                (*vtx.as_ptr()).predecessor = *el_v;
                            }
                            None => {}
                        }

                        match el_prev {
                            Some(vtx) => {
                                (*vtx.as_ptr()).successor = *el_v;
                            }
                            None => {}
                        }

                        if (*seg.as_ptr()).last.is_none() {
                            (*seg.as_ptr()).first = *el_v;
                        }
                        (*seg.as_ptr()).last = *el_v
                    }
                },
                None => {}
            }
        }
    }

    fn between(&self, from: &Self::TourNode, mid: &Self::TourNode, to: &Self::TourNode) -> bool {
        match (&from.segment, &mid.segment, &to.segment) {
            (Some(sf), Some(sm), Some(st)) => {
                match (sf == sm, sm == st, st == sf) {
                    (true, true, true) => unsafe {
                        (*sf.as_ptr()).reverse ^ between(from.rank, mid.rank, to.rank)
                    },
                    (true, false, false) => unsafe {
                        (*sf.as_ptr()).reverse ^ (from.rank <= mid.rank)
                    },
                    (false, true, false) => unsafe {
                        (*sm.as_ptr()).reverse ^ (mid.rank <= to.rank)
                    },
                    (false, false, true) => unsafe {
                        (*st.as_ptr()).reverse ^ (to.rank <= from.rank)
                    },
                    (false, false, false) => unsafe {
                        between(
                            (*sf.as_ptr()).rank,
                            (*sm.as_ptr()).rank,
                            (*st.as_ptr()).rank,
                        )
                    },
                    // (true, true, false)
                    // (true, false, true)
                    // (false, true, true)
                    _ => panic!("The transitivity requirement is violated."),
                }
            }
            _ => false,
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

    fn distance_at(&self, a: usize, b: usize) -> crate::Scalar {
        todo!()
    }

    fn flip_at(&mut self, from_a: usize, to_a: usize, from_b: usize, to_b: usize) {
        if let (Some(ofan), Some(otan), Some(ofbn), Some(otbn)) = (
            self.get_nn(from_a),
            self.get_nn(to_a),
            self.get_nn(from_b),
            self.get_nn(to_b),
        ) {
            match (ofan, otan, ofbn, otbn) {
                (Some(fan), Some(tan), Some(fbn), Some(tbn)) => unsafe {
                    match (
                        (*fan.as_ptr()).segment,
                        (*tan.as_ptr()).segment,
                        (*fbn.as_ptr()).segment,
                        (*tbn.as_ptr()).segment,
                    ) {
                        (Some(sfa), Some(sta), Some(sfb), Some(stb)) => {
                            if sfa == stb && (*tbn.as_ptr()).rank <= (*fan.as_ptr()).rank {
                                if ((*sfa.as_ptr()).first == *ofan
                                    && (*sfa.as_ptr()).reverse
                                    && (*sfa.as_ptr()).last == *otbn)
                                    || ((*sfa.as_ptr()).first == *otbn
                                        && (*sfa.as_ptr()).last == *ofan)
                                {
                                    return (*sfa.as_ptr()).reverse();
                                }
                                return reverse_segment(&sfa, &tbn, &fan);
                            } else if sfb == sta && (*tan.as_ptr()).rank <= (*fbn.as_ptr()).rank {
                                if ((*sfb.as_ptr()).first == *ofbn
                                    && (*sfb.as_ptr()).reverse
                                    && (*sfb.as_ptr()).last == *otan)
                                    || ((*sfb.as_ptr()).first == *otan
                                        && (*sfb.as_ptr()).last == *ofbn)
                                {
                                    return (*sfb.as_ptr()).reverse();
                                }
                                return reverse_segment(&sfb, &tan, &fbn);
                            }
                        }
                        _ => panic!("Node without segment while flipping."),
                    }
                },
                _ => panic!("Nullpointer"),
            }

            // TODO: better panic message.
        }
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&Self::TourNode> {
        match self.vertices.get(index) {
            Some(v) => match v {
                Some(n) => unsafe { Some(n.as_ref()) },
                None => None,
            },
            None => None,
        }
    }

    fn successor(&self, kin: &Self::TourNode) -> Option<&Self::TourNode> {
        todo!()
    }

    fn successor_at(&self, kin_index: usize) -> Option<&Self::TourNode> {
        match self.vertices.get(kin_index) {
            Some(kin) => match kin {
                Some(vtx) => unsafe {
                    match &(*vtx.as_ptr()).segment {
                        Some(seg) => {
                            if (*seg.as_ptr()).reverse {
                                match &(*vtx.as_ptr()).predecessor {
                                    Some(p) => Some(p.as_ref()),
                                    None => None,
                                }
                            } else {
                                match &(*vtx.as_ptr()).successor {
                                    Some(s) => Some(s.as_ref()),
                                    None => None,
                                }
                            }
                        }
                        None => None,
                    }
                },
                None => None,
            },
            None => None,
        }
    }

    fn predecessor(&self, kin: &Self::TourNode) -> Option<&Self::TourNode> {
        todo!()
    }

    fn predecessor_at(&self, kin_index: usize) -> Option<&Self::TourNode> {
        match self.vertices.get(kin_index) {
            Some(kin) => match kin {
                Some(vtx) => unsafe {
                    match &(*vtx.as_ptr()).segment {
                        Some(seg) => {
                            if (*seg.as_ptr()).reverse {
                                match &(*vtx.as_ptr()).successor {
                                    Some(s) => Some(s.as_ref()),
                                    None => None,
                                }
                            } else {
                                match &(*vtx.as_ptr()).predecessor {
                                    Some(p) => Some(p.as_ref()),
                                    None => None,
                                }
                            }
                        }
                        None => None,
                    }
                },
                None => None,
            },
            None => None,
        }
    }

    fn reset(&mut self) {
        todo!()
    }

    fn len(&self) -> usize {
        self.vertices.len()
    }

    fn total_distance(&self) -> crate::Scalar {
        todo!()
    }

    fn visited_at(&mut self, kin_index: usize, flag: bool) {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct TllNode {
    data: Node,
    rank: i32,
    segment: Option<NonNull<Segment>>,
    predecessor: Option<NonNull<TllNode>>,
    successor: Option<NonNull<TllNode>>,
}

impl TllNode {
    pub fn new(node: &Node) -> Self {
        Self {
            data: node.clone(),
            rank: i32::MAX,
            segment: None,
            predecessor: None,
            successor: None,
        }
    }
}

impl Vertex for TllNode {
    fn index(&self) -> usize {
        self.data.index()
    }

    fn is_visited(&self) -> bool {
        todo!()
    }

    fn visited(&mut self, flag: bool) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Segment {
    rank: usize,
    max_len: usize,
    reverse: bool,
    first: Option<NonNull<TllNode>>,
    last: Option<NonNull<TllNode>>,
    next: Option<NonNull<Segment>>,
    prev: Option<NonNull<Segment>>,
}

impl Segment {
    pub fn new(rank: usize, max_len: usize) -> Self {
        Self {
            rank,
            max_len,
            reverse: false,
            first: None,
            last: None,
            next: None,
            prev: None,
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.reverse = false;
        self.first = None;
        self.last = None;
    }

    #[inline]
    pub(super) fn reverse(&mut self) {
        // TODO: better panic message.
        match (&self.first, &self.last) {
            (Some(first), Some(last)) => unsafe {
                match (&(*first.as_ptr()).predecessor, &(*last.as_ptr()).successor) {
                    (Some(p), Some(s)) => {
                        if &(*p.as_ptr()).predecessor == &self.first {
                            (*p.as_ptr()).predecessor = self.last;
                        } else {
                            (*p.as_ptr()).successor = self.last;
                        }

                        if &(*s.as_ptr()).predecessor == &self.last {
                            (*s.as_ptr()).predecessor = self.first;
                        } else {
                            (*s.as_ptr()).successor = self.first;
                        }
                    }
                    _ => panic!("Empty predecessor or successor in node."),
                }
                let tmp = (*first.as_ptr()).predecessor;
                (*first.as_ptr()).predecessor = (*last.as_ptr()).successor;
                (*last.as_ptr()).successor = tmp;
            },
            _ => panic!("Empty first or last pointers in segment."),
        }
        self.reverse ^= true;
    }
}

#[inline]
fn to_nonnull<T>(x: T) -> Option<NonNull<T>> {
    let boxed = Box::new(x);
    Some(Box::leak(boxed).into())
}

// TODO: should be mut self.
// TODO: better panic msg.
unsafe fn reverse_segment(seg: &NonNull<Segment>, a: &NonNull<TllNode>, b: &NonNull<TllNode>) {
    let a_pred = (*a.as_ptr()).predecessor;
    let b_succ = (*b.as_ptr()).successor;
    (*a.as_ptr()).predecessor = b_succ;
    (*b.as_ptr()).successor = a_pred;

    let (rl, rr) = ((*a.as_ptr()).rank, (*b.as_ptr()).rank);
    let mut rank = rr;
    let mut node = *a;

    while rank >= rl {
        let tmp = (*node.as_ptr()).successor;
        (*node.as_ptr()).successor = (*node.as_ptr()).predecessor;
        (*node.as_ptr()).predecessor = tmp;
        (*node.as_ptr()).rank = rank;
        rank -= 1;

        match tmp {
            Some(next) => node = next,
            None => break,
        }
    }

    match a_pred {
        Some(pred) => {
            if (*pred.as_ptr()).predecessor == Some(*a) {
                (*pred.as_ptr()).predecessor = Some(*b);
            } else {
                (*pred.as_ptr()).successor = Some(*b);
            }
        }
        None => panic!("No predecessor when attempting to reverse segment."),
    }

    match b_succ {
        Some(succ) => {
            if (*succ.as_ptr()).predecessor == Some(*b) {
                (*succ.as_ptr()).predecessor = Some(*a);
            } else {
                (*succ.as_ptr()).successor = Some(*a);
            }
        }
        None => panic!("No predecessor when attempting to reverse segment."),
    }

    if (*seg.as_ptr()).first == Some(*a) {
        (*seg.as_ptr()).first = Some(*b);
    } else if (*seg.as_ptr()).first == Some(*b) {
        (*seg.as_ptr()).first = Some(*a);
    }

    if (*seg.as_ptr()).last == Some(*a) {
        (*seg.as_ptr()).last = Some(*b);
    } else if (*seg.as_ptr()).last == Some(*b) {
        (*seg.as_ptr()).last = Some(*a);
    }
}
