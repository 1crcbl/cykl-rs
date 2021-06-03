use std::{fmt::Display, ptr::NonNull};

use tspf::metric::MetricPoint;

use crate::{DataNode, Scalar};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TourNode {
    pub(super) inner: Option<NonNull<InnerNode>>,
}

impl TourNode {
    pub fn new(node: &DataNode) -> Self {
        let inner = InnerNode::new(node);
        Self {
            inner: to_nonnull(inner),
        }
    }

    pub fn empty() -> Self {
        Self { inner: None }
    }

    #[inline]
    pub fn visited(&mut self, flag: bool) {
        match self.inner {
            Some(inner) => unsafe {
                (*inner.as_ptr()).visited = flag;
            },
            None => {}
        }
    }

    #[inline]
    pub fn is_visisted(&self) -> bool {
        match self.inner {
            Some(inner) => unsafe { (*inner.as_ptr()).visited },
            None => false,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        match self.inner {
            Some(inner) => unsafe { inner.as_ref().data.index() },
            None => 0,
        }
    }

    #[inline]
    pub(super) fn data(&self) -> &DataNode {
        match self.inner {
            Some(inner) => unsafe { &(*inner.as_ptr()).data },
            None => panic!("Nullpointer"),
        }
    }

    pub fn set_candidates(&mut self, mut candidates: Vec<TourNode>) {
        match self.inner {
            Some(inner) => unsafe {
                (*inner.as_ptr()).candidates = Vec::with_capacity(candidates.len());
                (*inner.as_ptr()).candidates.append(&mut candidates);
            },
            None => panic!("Nullpointer"),
        }
    }

    pub fn candidates(&self) -> &Vec<TourNode> {
        match self.inner {
            Some(inner) => unsafe { &(*inner.as_ptr()).candidates },
            None => panic!("Nullpointer"),
        }
    }
}

impl Display for TourNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.inner {
            Some(inner) => write!(f, "TourNode: {}", format!("{:?}", inner)),
            None => write!(f, "{}", "None"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct InnerNode {
    pub(super) data: DataNode,
    /// Flag indicating whether a node is already visisted/processed by an algorithm.
    pub(super) visited: bool,
    /// The parent segment in a tour to which a node belongs.
    pub(super) segment: Option<NonNull<Segment>>,
    /// The rank of a node in its parent segment.
    pub(super) rank: i32,
    /// The directly preceding neighbour of a node in a tour.
    pub(super) predecessor: Option<NonNull<InnerNode>>,
    /// The directly succeeding neighbour of a node in a tour.
    pub(super) successor: Option<NonNull<InnerNode>>,
    /// Number of edges that are incident to the node.
    pub(super) degree: i32,
    /// Penalty value of a node in the ascent scheme. Corresponds to pi in LKH report.
    pub(super) penalty_weight: Scalar,
    /// Edge with minimum distance that doesn't belong to the MST.
    // TODO: better name
    pub(super) mst_final_edge: Option<NonNull<InnerNode>>,
    /// The parent of a node in a minimum spanning tree.
    pub(super) mst_parent: Option<NonNull<InnerNode>>,
    /// Set of candidate nodes.
    pub(super) candidates: Vec<TourNode>,
}

impl InnerNode {
    pub fn new(node: &DataNode) -> Self {
        Self {
            data: node.clone(),
            rank: i32::MAX,
            visited: false,
            segment: None,
            predecessor: None,
            successor: None,
            degree: 0,
            penalty_weight: 0.,
            mst_final_edge: None,
            mst_parent: None,
            candidates: Vec::with_capacity(0),
        }
    }

    #[inline]
    pub fn visited(&mut self, flag: bool) {
        self.visited = flag;
    }
}

impl Display for InnerNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "id: {} | x: {} | y: {} | z : {}",
                self.data.index(),
                self.data.x(),
                self.data.y(),
                self.data.z()
            )
        )
    }
}

#[derive(Debug)]
pub struct Segment {
    pub(super) rank: usize,
    pub(super) max_len: usize,
    pub(super) reverse: bool,
    pub(super) first: Option<NonNull<InnerNode>>,
    pub(super) last: Option<NonNull<InnerNode>>,
    pub(super) next: Option<NonNull<Segment>>,
    pub(super) prev: Option<NonNull<Segment>>,
}

macro_rules! move_node {
    ($s:expr, $target:ident, $kin1:ident, $kin2:ident, $reverse:expr, $head:ident, $tail:ident, $el_cnt:ident, $op:tt) => {
        match $s.$target {
            Some(target_node) => {
                let mut rank = 1;
                let target_rank = (*target_node.as_ptr()).rank;
                let seg = (*target_node.as_ptr()).segment;

                if $reverse {
                    let mut opt = $head;
                    while rank <= $el_cnt {
                        match opt {
                            Some(node) => {
                                opt = (*node.as_ptr()).$kin2;
                                (*node.as_ptr()).rank = target_rank $op rank;
                                (*node.as_ptr()).segment = seg;
                                std::mem::swap(&mut (*node.as_ptr()).successor, &mut (*node.as_ptr()).predecessor)
                            }
                            None => panic!("Nullpointer"),
                        }
                        rank += 1;
                    }
                    $s.$target = $tail;
                } else {
                    let mut opt = $tail;
                    while rank <= $el_cnt {
                        match opt {
                            Some(node) => {
                                opt = (*node.as_ptr()).$kin1;
                                (*node.as_ptr()).rank = target_rank $op rank;
                                (*node.as_ptr()).segment = seg;
                            }
                            None => panic!("Nullpointer"),
                        }
                        rank += 1;
                    }
                    $s.$target = $head;
                }
            }
            None => panic!("First not found"),
        }
    };
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
    pub fn reset(&mut self) {
        self.reverse = false;
        self.first = None;
        self.last = None;
        self.next = None;
        self.prev = None;
        self.rank = 0;
    }

    #[inline]
    pub unsafe fn reverse(&mut self) {
        // TODO: better panic message.
        match (&self.first, &self.last) {
            (Some(first), Some(last)) => {
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
                std::mem::swap(
                    &mut (*first.as_ptr()).predecessor,
                    &mut (*last.as_ptr()).successor,
                );
            }
            _ => panic!("Empty first or last pointers in segment."),
        }
        self.reverse ^= true;
    }

    pub(super) unsafe fn split(&mut self, node: &NonNull<InnerNode>) {
        match (self.first, self.last) {
            (Some(first), Some(last)) => {
                let (f1, f2) = if self.reverse { (1, 0) } else { (0, 1) };

                let d1 = (*node.as_ptr()).rank - (*first.as_ptr()).rank + f1;
                let d2 = (*last.as_ptr()).rank - (*node.as_ptr()).rank + f2;

                if d1 <= d2 {
                    if self.reverse {
                        let tmp_ptr = (*node.as_ptr()).successor;
                        match self.next {
                            Some(next) => {
                                (*next.as_ptr()).move_front(
                                    self.first,
                                    Some(*node),
                                    d1,
                                    self.reverse,
                                );
                            }
                            None => panic!("No next"),
                        }
                        self.first = tmp_ptr;
                    } else {
                        match self.prev {
                            Some(prev) => {
                                (*prev.as_ptr()).move_back(
                                    self.first,
                                    (*node.as_ptr()).predecessor,
                                    d1,
                                    self.reverse,
                                );
                            }
                            None => panic!("No prev"),
                        }
                        self.first = Some(*node);
                    }
                } else {
                    if self.reverse {
                        match self.prev {
                            Some(prev) => {
                                (*prev.as_ptr()).move_back(
                                    (*node.as_ptr()).successor,
                                    self.last,
                                    d2,
                                    self.reverse,
                                );
                            }
                            None => panic!("No prev"),
                        }
                        self.last = Some(*node);
                    } else {
                        let tmp_ptr = (*node.as_ptr()).predecessor;
                        match self.next {
                            Some(next) => {
                                (*next.as_ptr()).move_front(
                                    Some(*node),
                                    self.last,
                                    d2,
                                    self.reverse,
                                );
                            }
                            None => panic!("No next"),
                        }
                        self.last = tmp_ptr;
                    }
                }
            }
            _ => panic!("Missing first/last"),
        }
    }

    pub(super) unsafe fn move_back(
        &mut self,
        head: Option<NonNull<InnerNode>>,
        tail: Option<NonNull<InnerNode>>,
        el_cnt: i32,
        reverse: bool,
    ) {
        if self.reverse {
            move_node!(self, first, predecessor, successor, !reverse, head, tail, el_cnt, -);
        } else {
            move_node!(self, last, successor, predecessor, reverse, tail, head, el_cnt, +);
        }
    }

    pub(super) unsafe fn move_front(
        &mut self,
        head: Option<NonNull<InnerNode>>,
        tail: Option<NonNull<InnerNode>>,
        el_cnt: i32,
        reverse: bool,
    ) {
        if self.reverse {
            move_node!(self, last, successor, predecessor, !reverse, tail, head, el_cnt, +);
        } else {
            move_node!(self, first, predecessor, successor, reverse, head, tail, el_cnt, -);
        }
    }
}

#[inline]
pub fn to_nonnull<T>(x: T) -> Option<NonNull<T>> {
    let boxed = Box::new(x);
    Some(Box::leak(boxed).into())
}

macro_rules! change_kin {
    ($target:ident, $cond_kin:ident, $new_kin:ident) => {
        match $target {
            Some(node) => {
                if (*node.as_ptr()).predecessor == Some(*$cond_kin) {
                    (*node.as_ptr()).predecessor = Some(*$new_kin);
                } else {
                    (*node.as_ptr()).successor = Some(*$new_kin);
                }
            }
            None => panic!("No predecessor when attempting to reverse segment."),
        }
    };
}

/// Reverse a segment internally.
// TODO: better panic msg.
pub(super) unsafe fn reverse_int_seg(
    seg: &NonNull<Segment>,
    a: &NonNull<InnerNode>,
    b: &NonNull<InnerNode>,
) {
    let a_pred = (*a.as_ptr()).predecessor;
    let b_succ = (*b.as_ptr()).successor;
    (*a.as_ptr()).predecessor = b_succ;
    (*b.as_ptr()).successor = a_pred;

    let (rl, rr) = ((*a.as_ptr()).rank, (*b.as_ptr()).rank);
    let mut rank = rr;
    let mut node = *a;

    while rank >= rl {
        let tmp = (*node.as_ptr()).successor;
        std::mem::swap(
            &mut (*node.as_ptr()).successor,
            &mut (*node.as_ptr()).predecessor,
        );
        (*node.as_ptr()).rank = rank;
        rank -= 1;

        match tmp {
            Some(next) => node = next,
            None => break,
        }
    }

    change_kin!(a_pred, a, b);
    change_kin!(b_succ, b, a);

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

macro_rules! swap_sym {
    ($head: ident, $tail:ident, $target:ident, $key:ident) => {
        match ((*$head.as_ptr()).$target, (*$tail.as_ptr()).$target) {
            (Some(node_a), Some(node_b)) => {
                let kin_from_a = (*node_a.as_ptr()).$key;
                let kin_from_b = (*node_b.as_ptr()).$key;

                match kin_from_a {
                    Some(kin) => {
                        if (*kin.as_ptr()).predecessor == (*$head.as_ptr()).$target {
                            (*kin.as_ptr()).predecessor = (*$tail.as_ptr()).$target;
                        } else {
                            (*kin.as_ptr()).successor = (*$tail.as_ptr()).$target;
                        }
                    }
                    None => panic!("Missing pointer"),
                }

                match kin_from_b {
                    Some(kin) => {
                        if (*kin.as_ptr()).predecessor == (*$tail.as_ptr()).$target {
                            (*kin.as_ptr()).predecessor = (*$head.as_ptr()).$target;
                        } else {
                            (*kin.as_ptr()).successor = (*$head.as_ptr()).$target;
                        }
                    }
                    None => panic!("Missing pointer"),
                }

                (*node_a.as_ptr()).$key = kin_from_b;
                (*node_b.as_ptr()).$key = kin_from_a;
            }
            _ => panic!("Missing pointers"),
        }
    };
}

macro_rules! swap_asym {
    ($head:ident, $tail:ident) => {
        match ((*$head.as_ptr()).first, (*$tail.as_ptr()).last) {
            (Some(node_a), Some(node_b)) => {
                let a_pred = (*node_a.as_ptr()).predecessor;
                let b_succ = (*node_b.as_ptr()).successor;

                match a_pred {
                    Some(pred) => {
                        if (*pred.as_ptr()).predecessor == (*$head.as_ptr()).first {
                            (*pred.as_ptr()).predecessor = (*$tail.as_ptr()).last;
                        } else {
                            (*pred.as_ptr()).successor = (*$tail.as_ptr()).last;
                        }
                    }
                    None => panic!("Missing pointer"),
                }

                match b_succ {
                    Some(succ) => {
                        if (*succ.as_ptr()).predecessor == (*$tail.as_ptr()).last {
                            (*succ.as_ptr()).predecessor = (*$head.as_ptr()).first;
                        } else {
                            (*succ.as_ptr()).successor = (*$head.as_ptr()).first;
                        }
                    }
                    None => panic!("Missing pointer"),
                }

                (*node_a.as_ptr()).predecessor = b_succ;
                (*node_b.as_ptr()).successor = a_pred;
            }
            _ => panic!("Missing pointers"),
        }
    };
}

// TODO: better panic msg.
// TODO: this fn is a bomb. needs an intensive care.
pub unsafe fn reverse_segs(from: &NonNull<Segment>, to: &NonNull<Segment>) {
    let mut a = *from;
    let mut b = *to;

    loop {
        if a == b {
            (*a.as_ptr()).reverse();
            break;
        }

        match ((*a.as_ptr()).reverse, (*b.as_ptr()).reverse) {
            (true, true) | (false, false) => {
                swap_sym!(a, b, first, predecessor);
                swap_sym!(a, b, last, successor);
            }
            (true, false) | (false, true) => {
                swap_asym!(a, b);
                swap_asym!(b, a);
            }
        }

        (*a.as_ptr()).reverse();
        (*b.as_ptr()).reverse();

        let tmp_next = (*a.as_ptr()).next;
        let tmp_prev = (*b.as_ptr()).prev;

        std::mem::swap(&mut (*a.as_ptr()).rank, &mut (*b.as_ptr()).rank);

        match ((*a.as_ptr()).prev, (*a.as_ptr()).next) {
            (Some(prev), Some(next)) => {
                (*prev.as_ptr()).next = Some(b);
                (*next.as_ptr()).prev = Some(b);
            }
            _ => panic!("Nullpointer"),
        };

        match ((*b.as_ptr()).prev, (*b.as_ptr()).next) {
            (Some(prev), Some(next)) => {
                (*prev.as_ptr()).next = Some(a);
                (*next.as_ptr()).prev = Some(a);
            }
            _ => panic!("Nullpointer"),
        };

        (*a.as_ptr()).next = (*b.as_ptr()).next;
        (*b.as_ptr()).prev = (*a.as_ptr()).prev;

        if tmp_next == Some(b) {
            // a is the neighbour directly in front of b.
            (*b.as_ptr()).next = Some(a);
            (*a.as_ptr()).prev = Some(b);
            break;
        } else {
            (*b.as_ptr()).next = tmp_next;
            (*a.as_ptr()).prev = tmp_prev;
        }

        match tmp_next {
            Some(next) => a = next,
            None => panic!("Missing next segment"),
        }

        match tmp_prev {
            Some(prev) => b = prev,
            None => panic!("Missing prev segment"),
        }
    }
}
