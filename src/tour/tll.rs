use std::ptr::NonNull;

use crate::{tour::HeldKarpBound, DataNode, Repo, Scalar};

use super::{between, STree, Tour, TourIter, TourOrder, Vertex};

#[derive(Debug)]
pub struct TwoLevelList<'a> {
    repo: &'a Repo,
    pub(crate) segments: Vec<Option<NonNull<Segment>>>,
    nodes: Vec<Option<NonNull<TllNode>>>,
    total_dist: Scalar,
}

impl<'a> TwoLevelList<'a> {
    pub fn new(repo: &'a Repo, max_grouplen: usize) -> Self {
        let mut n_segments = repo.size() / max_grouplen;
        if repo.size() % max_grouplen != 0 {
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

        let nodes = repo
            .into_iter()
            .map(|node| to_nonnull(TllNode::new(node)))
            .collect();

        Self {
            repo,
            nodes: nodes,
            segments: segments,
            total_dist: 0.,
        }
    }

    pub fn with_default_order(repo: &'a Repo, max_grouplen: usize) -> Self {
        let mut result = Self::new(repo, max_grouplen);
        result.apply(&TourOrder::new((0..repo.size()).collect()));
        result
    }
}

impl<'a> Tour for TwoLevelList<'a> {
    type TourNode = TllNode;

    fn apply(&mut self, tour: &super::TourOrder) {
        let order = tour.order();
        let v_len = self.nodes.len();
        let s_len = self.segments.len();

        self.total_dist = 0.;
        for (sidx, els) in self.segments.iter().enumerate() {
            match els {
                Some(seg) => unsafe {
                    (*seg.as_ptr()).reset();
                    (*seg.as_ptr()).rank = sidx;
                    (*seg.as_ptr()).next = self.segments[(sidx + 1) % s_len];
                    (*seg.as_ptr()).prev = self.segments[(s_len + sidx - 1) % s_len];

                    let max_len = seg.as_ref().max_len;
                    let beg_seg = sidx * max_len;
                    let end_seg = (beg_seg + max_len).min(v_len);

                    for iv in beg_seg..end_seg {
                        let el_v = self.nodes.get(order[iv]).unwrap();
                        let el_next = self.nodes.get(order[(iv + 1) % v_len]).unwrap();
                        let el_prev = self.nodes.get(order[(v_len + iv - 1) % v_len]).unwrap();

                        match (el_v, el_next, el_prev) {
                            (Some(vtx), Some(vtx_nxt), Some(vtx_prv)) => {
                                (*vtx.as_ptr()).predecessor = *el_prev;
                                (*vtx.as_ptr()).successor = *el_next;
                                (*vtx.as_ptr()).rank = (iv - beg_seg) as i32;
                                (*vtx.as_ptr()).segment = *els;

                                (*vtx_nxt.as_ptr()).predecessor = *el_v;
                                (*vtx_prv.as_ptr()).successor = *el_v;

                                self.total_dist += self
                                    .repo
                                    .distance(&(*vtx.as_ptr()).data, &(*vtx_nxt.as_ptr()).data);
                            }
                            _ => panic!("Nodes not found"),
                        }

                        if (*seg.as_ptr()).last.is_none() {
                            (*seg.as_ptr()).first = *el_v;
                        }
                        (*seg.as_ptr()).last = *el_v;
                    }
                },
                None => panic!("Segment not found"),
            }
        }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    fn distance(&self, a: &Self::TourNode, b: &Self::TourNode) -> Scalar {
        self.repo.distance(&a.data, &b.data)
    }

    #[inline]
    fn distance_at(&self, a: usize, b: usize) -> crate::Scalar {
        self.repo.distance_at(a, b)
    }

    fn flip_at(&mut self, from_a: usize, to_a: usize, from_b: usize, to_b: usize) {
        if let (Some(ofan), Some(otan), Some(ofbn), Some(otbn)) = (
            self.nodes.get(from_a),
            self.nodes.get(to_a),
            self.nodes.get(from_b),
            self.nodes.get(to_b),
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
                            // Case 1: Either the entire path (to_b, from_a) or (to_a, from_b)
                            // resides in the same segment. In this case, we will flip either the
                            // local path or the entire segment if both nodes are the end nodes
                            // of that segment.
                            if sfa == stb && (*tbn.as_ptr()).rank <= (*fan.as_ptr()).rank {
                                if ((*sfa.as_ptr()).first == *ofan
                                    && (*sfa.as_ptr()).reverse
                                    && (*sfa.as_ptr()).last == *otbn)
                                    || ((*sfa.as_ptr()).first == *otbn
                                        && (*sfa.as_ptr()).last == *ofan)
                                {
                                    return (*sfa.as_ptr()).reverse();
                                }
                                return reverse_int_seg(&sfa, &tbn, &fan);
                            } else if sfb == sta && (*tan.as_ptr()).rank <= (*fbn.as_ptr()).rank {
                                if ((*sfb.as_ptr()).first == *ofbn
                                    && (*sfb.as_ptr()).reverse
                                    && (*sfb.as_ptr()).last == *otan)
                                    || ((*sfb.as_ptr()).first == *otan
                                        && (*sfb.as_ptr()).last == *ofbn)
                                {
                                    return (*sfb.as_ptr()).reverse();
                                }
                                return reverse_int_seg(&sfb, &tan, &fbn);
                            }

                            // Case 2: Both paths (to_b, from_a) AND (to_a, from_b) consist of a
                            // sequence of consecutive segments. Since to_a and to_b are direct
                            // successors of from_a and from_b, this means that all nodes are
                            // either at the head or the tail of their corresponding segments.
                            // Thus, we only need to reverse these segments.
                            //
                            // Case 1 and 2 are special arrangements of nodes in the tour. A more
                            // general case is when nodes are positioned somewhere in the middle
                            // of their segments. To tackle this case, we will rearrange affected
                            // nodes by splitting their corresponding segments so that the
                            // requirements for case 1 or 2 are satisfied.

                            // Check for case 3.
                            let mut split = false;
                            if sfa == sta {
                                // split a
                                split = true;
                                (*sfa.as_ptr()).split(tan);
                            }

                            if sfb == stb {
                                // split b
                                split = true;
                                (*sfb.as_ptr()).split(tbn);
                            }

                            if split {
                                return self.flip_at(from_a, to_a, from_b, to_b);
                            }

                            // Logic to handle case 2.
                            let (sfa_r, sta_r, sfb_r, stb_r) = (
                                (*sfa.as_ptr()).rank,
                                (*sta.as_ptr()).rank,
                                (*sfb.as_ptr()).rank,
                                (*stb.as_ptr()).rank,
                            );

                            let diff1 = if sta_r <= sfb_r {
                                sfb_r - sta_r
                            } else {
                                self.segments.len() - sta_r + sfb_r
                            };

                            let diff2 = if stb_r <= sfa_r {
                                sfa_r - stb_r
                            } else {
                                self.segments.len() - stb_r + sfa_r
                            };

                            if diff1 <= diff2 {
                                // Reverses the path (to_a, from_b).
                                return reverse_segs(&sta, &sfb);
                            } else {
                                // Reverses the path (to_b, from_a).
                                return reverse_segs(&stb, &sfa);
                            };
                        }
                        _ => panic!("DataNode without segment while flipping."),
                    }
                },
                _ => panic!("Nullpointer"),
            }

            // TODO: better panic message.
        }
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&Self::TourNode> {
        match self.nodes.get(index) {
            Some(v) => match v {
                Some(n) => unsafe { Some(n.as_ref()) },
                None => None,
            },
            None => None,
        }
    }

    #[inline]
    fn successor(&self, node: &Self::TourNode) -> Option<&Self::TourNode> {
        match &node.segment {
            Some(seg) => unsafe {
                if (*seg.as_ptr()).reverse {
                    match &node.predecessor {
                        Some(p) => Some(&(*p.as_ptr())),
                        None => panic!("No predecessor"),
                    }
                } else {
                    match node.successor {
                        Some(s) => Some(&(*s.as_ptr())),
                        None => None,
                    }
                }
            },
            None => None,
        }
    }

    #[inline]
    fn successor_at(&self, kin_index: usize) -> Option<&Self::TourNode> {
        match self.nodes.get(kin_index) {
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

    #[inline]
    fn predecessor(&self, node: &Self::TourNode) -> Option<&Self::TourNode> {
        match &node.segment {
            Some(seg) => unsafe {
                if (*seg.as_ptr()).reverse {
                    match &node.successor {
                        Some(s) => Some(&(*s.as_ptr())),
                        None => None,
                    }
                } else {
                    match node.predecessor {
                        Some(p) => Some(&(*p.as_ptr())),
                        None => None,
                    }
                }
            },
            None => None,
        }
    }

    #[inline]
    fn predecessor_at(&self, kin_index: usize) -> Option<&Self::TourNode> {
        match self.nodes.get(kin_index) {
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

    #[inline]
    fn len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    fn total_distance(&self) -> crate::Scalar {
        self.total_dist
    }

    // TODO: better panic message.
    #[inline]
    fn visited_at(&mut self, node_index: usize, flag: bool) {
        match self.nodes.get(node_index) {
            Some(opt) => match opt {
                Some(node) => unsafe {
                    (*node.as_ptr()).visited(flag);
                },
                None => panic!("Missing pointer."),
            },
            None => {}
        }
    }

    fn gen_cands(&mut self, k: usize) {
        let len = self.nodes.len();
        for (base_idx, base) in self.nodes.iter().enumerate() {
            let mut targ_idx = (base_idx + 1) % len;
            let mut cands = vec![None; k];
            let mut cands_d = vec![Scalar::MAX; k];
            let mut count = 0;

            while targ_idx != base_idx {
                let targ = &self.nodes[targ_idx];

                match (base, targ) {
                    (Some(nb), Some(nt)) => unsafe {
                        if count < k {
                            count += 1;
                        }
                        let mut c_idx = count - 1;

                        let d = self.distance(nb.as_ref(), nt.as_ref());

                        while c_idx > 0 && d < cands_d[c_idx - 1] {
                            cands[c_idx] = cands[c_idx - 1];
                            cands_d[c_idx] = cands_d[c_idx - 1];
                            c_idx -= 1;
                        }

                        if d < cands_d[c_idx] {
                            cands[c_idx] = *targ;
                            cands_d[c_idx] = d;
                        }
                    },
                    _ => panic!("Nullpointers"),
                }

                targ_idx = (targ_idx + 1) % len;
            }

            unsafe {
                (*base.unwrap().as_ptr()).cands = cands;
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TllNode {
    data: DataNode,
    /// Flag indicating whether a node is already visisted/processed by an algorithm.
    visited: bool,
    /// The parent segment in a tour to which a node belongs.
    segment: Option<NonNull<Segment>>,
    /// The rank of a node in its parent segment.
    rank: i32,
    /// The directly preceding neighbour of a node in a tour.
    predecessor: Option<NonNull<TllNode>>,
    /// The directly succeeding neighbour of a node in a tour.
    successor: Option<NonNull<TllNode>>,
    /// Number of edges that are incident to the node.
    degree: i32,
    /// Penalty value of a node in the ascent scheme. Corresponds to pi in LKH report.
    penalty_weight: Scalar,
    /// Edge with minimum distance that doesn't belong to the MST.
    // TODO: better name
    mst_final_edge: Option<NonNull<TllNode>>,
    /// The parent of a node in a minimum spanning tree.
    pub(super) mst_parent: Option<NonNull<TllNode>>,
    /// Set of candidate nodes.
    pub(super) cands: Vec<Option<NonNull<TllNode>>>,
}

impl TllNode {
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
            cands: Vec::with_capacity(0),
        }
    }
}

impl Vertex for TllNode {
    #[inline]
    fn index(&self) -> usize {
        self.data.index()
    }

    #[inline]
    fn is_visited(&self) -> bool {
        self.visited
    }

    #[inline]
    fn visited(&mut self, flag: bool) {
        self.visited = flag;
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
                                let tmp_ptr = (*node.as_ptr()).successor;
                                (*node.as_ptr()).successor = (*node.as_ptr()).predecessor;
                                (*node.as_ptr()).predecessor = tmp_ptr;
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
    fn reset(&mut self) {
        self.reverse = false;
        self.first = None;
        self.last = None;
        self.next = None;
        self.prev = None;
        self.rank = 0;
    }

    #[inline]
    unsafe fn reverse(&mut self) {
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
                let tmp = (*first.as_ptr()).predecessor;
                (*first.as_ptr()).predecessor = (*last.as_ptr()).successor;
                (*last.as_ptr()).successor = tmp;
            }
            _ => panic!("Empty first or last pointers in segment."),
        }
        self.reverse ^= true;
    }

    unsafe fn split(&mut self, node: &NonNull<TllNode>) {
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

    unsafe fn move_back(
        &mut self,
        head: Option<NonNull<TllNode>>,
        tail: Option<NonNull<TllNode>>,
        el_cnt: i32,
        reverse: bool,
    ) {
        if self.reverse {
            move_node!(self, first, predecessor, successor, !reverse, head, tail, el_cnt, -);
        } else {
            move_node!(self, last, successor, predecessor, reverse, tail, head, el_cnt, +);
        }
    }

    unsafe fn move_front(
        &mut self,
        head: Option<NonNull<TllNode>>,
        tail: Option<NonNull<TllNode>>,
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
fn to_nonnull<T>(x: T) -> Option<NonNull<T>> {
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
unsafe fn reverse_int_seg(seg: &NonNull<Segment>, a: &NonNull<TllNode>, b: &NonNull<TllNode>) {
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
unsafe fn reverse_segs(from: &NonNull<Segment>, to: &NonNull<Segment>) {
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

        let tmpr = (*a.as_ptr()).rank;
        (*a.as_ptr()).rank = (*b.as_ptr()).rank;
        (*b.as_ptr()).rank = tmpr;

        let tmp_next = (*a.as_ptr()).next;
        let tmp_prev = (*b.as_ptr()).prev;

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

impl<'a> STree for TwoLevelList<'a> {
    fn build_mst(&mut self) {
        // A naive implementation of Prim's algorithm. Runtime is O(N^2).
        // https://en.wikipedia.org/wiki/Prim%27s_algorithm
        let n_nodes = self.nodes.len();
        let mut selected = vec![false; n_nodes];
        let mut processed = 0;

        for nopt in &self.nodes {
            match nopt {
                Some(node) => unsafe {
                    (*node.as_ptr()).mst_parent = None;
                    (*node.as_ptr()).mst_final_edge = None;
                },
                None => panic!("Nullpointer"),
            }
        }

        selected[0] = true;

        while processed != n_nodes - 1 {
            let (mut v_cand, mut w_cand, mut cost_cand) = (0, 0, Scalar::MAX);
            for (v_idx, v_sel) in selected.iter().enumerate() {
                if *v_sel {
                    for (w_idx, w_sel) in selected.iter().enumerate() {
                        // The edge (v, w) is forbidden if its cost is equal to 0.
                        let c = self.distance_at(v_idx, w_idx);
                        if !*w_sel && c > 0. && c < cost_cand {
                            v_cand = v_idx;
                            w_cand = w_idx;
                            cost_cand = c;
                        }
                    }
                }
            }

            let (vo, wo) = (self.nodes.get(v_cand), self.nodes.get(w_cand));
            match (vo, wo) {
                (Some(v), Some(w)) => match w {
                    Some(vtx) => unsafe {
                        (*vtx.as_ptr()).mst_parent = *v;
                    },
                    None => panic!("Nullpointer"),
                },
                _ => panic!("Nodes not found"),
            }

            selected[w_cand] = true;

            processed += 1;
        }
    }

    // Held-Karp lower bound
    fn cost_m1t(&self) -> HeldKarpBound {
        let (mut result, mut len_tree) = (0., 0.);
        for nopt in &self.nodes {
            match nopt {
                Some(node) => unsafe {
                    len_tree += (*node.as_ptr()).penalty_weight;
                    (*node.as_ptr()).degree -= 2;

                    match (*node.as_ptr()).mst_parent {
                        Some(parent) => {
                            (*node.as_ptr()).degree += 1;
                            (*parent.as_ptr()).degree += 1;
                            result += self.distance(node.as_ref(), parent.as_ref());
                        }
                        None => panic!("Nullpointer in mst_parent"),
                    }
                },
                None => panic!("Nullpointer"),
            }
        }

        let mut cand_d = Scalar::MAX;
        let mut cand = None;

        for nopt in &self.nodes {
            match nopt {
                Some(node) => unsafe {
                    if (*node.as_ptr()).degree == -1 {
                        for other_opt in &self.nodes {
                            match other_opt {
                                Some(other) => {
                                    if node == other
                                        || (*node.as_ptr()).mst_parent == *other_opt
                                        || (*other.as_ptr()).mst_parent == *nopt
                                    {
                                        continue;
                                    }

                                    let d = self.distance(node.as_ref(), other.as_ref());
                                    if d < cand_d {
                                        cand_d = d;
                                        (*node.as_ptr()).mst_final_edge = *other_opt;
                                        cand = *nopt;
                                    }
                                }
                                None => panic!("Nullpointer in mst_parent"),
                            }
                        }
                    }
                },
                None => {
                    panic!("Nullpointer")
                }
            }
        }

        match cand {
            Some(node) => unsafe {
                match (*node.as_ptr()).mst_parent {
                    Some(parent) => {
                        (*node.as_ptr()).degree += 1;
                        (*parent.as_ptr()).degree += 1;
                    }
                    None => panic!("No mst parent"),
                }
            },
            None => panic!("Nullpointer"),
        }

        let mut total_deg = 0;
        for nopt in &self.nodes {
            match nopt {
                Some(node) => unsafe {
                    total_deg += (*node.as_ptr()).degree * (*node.as_ptr()).degree;
                },
                None => panic!("Nullpointer"),
            }
        }

        if total_deg == 0 {
            HeldKarpBound::Optimal
        } else {
            result += len_tree * 2. + cand_d;
            HeldKarpBound::Value(result)
        }
    }
}

impl<'a, 's> TourIter<'s> for TwoLevelList<'a> {
    type Iter = TllIter<'s>;

    fn itr(&'s self) -> Self::Iter {
        TllIter {
            it: self.nodes.iter(),
        }
    }
}

pub struct TllIter<'s> {
    it: std::slice::Iter<'s, Option<NonNull<TllNode>>>,
}

impl<'s> Iterator for TllIter<'s> {
    type Item = &'s TllNode;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.it.next() {
            Some(opt) => unsafe {
                match opt {
                    Some(node) => Some(node.as_ref()),
                    None => None,
                }
            },
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.it.len(), Some(self.it.len()))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        match self.it.next_back() {
            Some(opt) => unsafe {
                match opt {
                    Some(node) => Some(node.as_ref()),
                    None => None,
                }
            },
            None => None,
        }
    }
}
