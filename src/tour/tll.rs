use std::ptr::NonNull;

use crate::{
    data::{DataStore, GetIndex, NodeStore},
    tour::{
        node::{reverse_int_seg, reverse_segs},
        NodeStatus,
    },
    Scalar,
};

use super::{
    between,
    node::{to_nonnull, Segment},
    NodeRel, Tour, TourIter, TourNode, TourOrder, UpdateTourError,
};

#[derive(Debug)]
pub struct TwoLevelList {
    store: NodeStore,
    pub(crate) segments: Vec<Option<NonNull<Segment>>>,
    nodes: Vec<TourNode>,
    total_dist: Scalar,
    rev: bool,
}

impl TwoLevelList {
    pub fn new<M>(store: &DataStore<M>, groupsize: usize) -> Self {
        let node_store = store.store();
        let n_nodes = store.len();

        let mut n_segments = n_nodes / groupsize;
        if n_nodes % groupsize != 0 {
            n_segments += 1;
        }

        let mut segments = Vec::with_capacity(n_segments);
        segments.push(to_nonnull(Segment::new(0, groupsize)));

        for ii in 1..n_segments {
            let s = to_nonnull(Segment::new(ii, groupsize));

            if let Some(el) = segments.last() {
                if let Some(last) = el {
                    unsafe {
                        (*s.unwrap().as_ptr()).prev = *el;
                        (*last.as_ptr()).next = s;
                    }
                }
            }

            if ii == n_segments - 1 {
                if let Some(el) = segments.first() {
                    if let Some(first) = el {
                        unsafe {
                            (*s.unwrap().as_ptr()).next = *el;
                            (*first.as_ptr()).prev = s;
                        }
                    }
                }
            }

            segments.push(s);
        }

        let nodes = node_store
            .into_iter()
            .map(|node| TourNode::new(*node))
            .collect();

        let mut result = Self {
            store: node_store,
            nodes,
            segments,
            total_dist: 0.,
            rev: false,
        };

        result
            .apply(&TourOrder::with_ord((0..n_nodes).collect()))
            .unwrap();

        result
    }
}

impl Tour for TwoLevelList {
    fn apply(&mut self, tour: &super::TourOrder) -> Result<(), UpdateTourError> {
        self.rev = false;
        let order = tour.order();
        let v_len = self.nodes.len();
        let s_len = self.segments.len();

        if order.len() != v_len {
            return Err(UpdateTourError::TourLenMismatched {
                expected: v_len,
                received: order.len(),
            });
        }

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

                        match (el_v.inner, el_next.inner, el_prev.inner) {
                            (Some(vtx), Some(vtx_nxt), Some(vtx_prv)) => {
                                (*vtx.as_ptr()).predecessor = el_prev.inner;
                                (*vtx.as_ptr()).successor = el_next.inner;
                                (*vtx.as_ptr()).rank = (iv - beg_seg) as i32;
                                (*vtx.as_ptr()).segment = *els;
                                (*vtx.as_ptr()).status = NodeStatus::Active;

                                (*vtx_nxt.as_ptr()).predecessor = el_v.inner;
                                (*vtx_prv.as_ptr()).successor = el_v.inner;

                                self.total_dist += self
                                    .store
                                    .cost(&(*vtx.as_ptr()).index, &(*vtx_nxt.as_ptr()).index);
                            }
                            _ => panic!("Nodes not found"),
                        }

                        if (*seg.as_ptr()).last.is_none() {
                            (*seg.as_ptr()).first = el_v.inner;
                        }
                        (*seg.as_ptr()).last = el_v.inner;
                    }
                },
                None => panic!("Segment not found"),
            }
        }

        Ok(())
    }

    #[inline]
    fn between(&self, from: &TourNode, mid: &TourNode, to: &TourNode) -> bool {
        match (from.inner, mid.inner, to.inner) {
            (Some(f), Some(m), Some(t)) => unsafe {
                match (
                    (*f.as_ptr()).segment,
                    (*m.as_ptr()).segment,
                    (*t.as_ptr()).segment,
                ) {
                    (Some(sf), Some(sm), Some(st)) => {
                        match (sf == sm, sm == st, st == sf) {
                            (true, true, true) => {
                                (*sf.as_ptr()).reverse
                                    ^ between(
                                        (*f.as_ptr()).rank,
                                        (*m.as_ptr()).rank,
                                        (*t.as_ptr()).rank,
                                    )
                            }
                            (true, false, false) => {
                                (*sf.as_ptr()).reverse ^ ((*f.as_ptr()).rank <= (*m.as_ptr()).rank)
                            }
                            (false, true, false) => {
                                (*sm.as_ptr()).reverse ^ ((*m.as_ptr()).rank <= (*t.as_ptr()).rank)
                            }
                            (false, false, true) => {
                                (*st.as_ptr()).reverse ^ ((*t.as_ptr()).rank <= (*f.as_ptr()).rank)
                            }
                            (false, false, false) => between(
                                (*sf.as_ptr()).rank,
                                (*sm.as_ptr()).rank,
                                (*st.as_ptr()).rank,
                            ),
                            // (true, true, false)
                            // (true, false, true)
                            // (false, true, true)
                            _ => panic!("The transitivity requirement is violated."),
                        }
                    }
                    _ => false,
                }
            },
            _ => panic!("Empty node"),
        }
    }

    #[inline]
    fn between_at(&self, from_index: usize, mid_index: usize, to_index: usize) -> bool {
        match (
            self.get(from_index),
            self.get(mid_index),
            self.get(to_index),
        ) {
            (Some(from), Some(mid), Some(to)) => self.between(&from, &mid, &to),
            _ => false,
        }
    }

    #[inline]
    fn distance(&self, a: &TourNode, b: &TourNode) -> Scalar {
        match (a.inner, b.inner) {
            (Some(ai), Some(bi)) => unsafe {
                self.store
                    .cost(&(*ai.as_ptr()).index, &(*bi.as_ptr()).index)
            },
            _ => 0.,
        }
    }

    #[inline]
    fn distance_at<I>(&self, a: &I, b: &I) -> crate::Scalar
    where
        I: GetIndex + PartialEq + Eq,
    {
        self.store.cost(a, b)
    }

    #[inline]
    fn flip_at(&mut self, from_a: usize, to_a: usize, from_b: usize, to_b: usize) {
        if let (Some(fa), Some(ta), Some(fb), Some(tb)) = (
            self.get(from_a),
            self.get(to_a),
            self.get(from_b),
            self.get(to_b),
        ) {
            self.flip(&fa, &ta, &fb, &tb);
        }
    }

    fn flip(&mut self, from_a: &TourNode, to_a: &TourNode, from_b: &TourNode, to_b: &TourNode) {
        match (from_a.inner, to_a.inner, from_b.inner, to_b.inner) {
            (Some(mut fan), Some(mut tan), Some(mut fbn), Some(mut tbn)) => unsafe {
                if self.rev {
                    std::mem::swap(&mut fan, &mut tan);
                    std::mem::swap(&mut fbn, &mut tbn);
                }

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
                            if ((*sfa.as_ptr()).first == from_a.inner
                                && (*sfa.as_ptr()).reverse
                                && (*sfa.as_ptr()).last == to_b.inner)
                                || ((*sfa.as_ptr()).first == to_b.inner
                                    && (*sfa.as_ptr()).last == from_a.inner)
                            {
                                return (*sfa.as_ptr()).reverse();
                            }
                            return reverse_int_seg(&sfa, &tbn, &fan);
                        } else if sfb == sta && (*tan.as_ptr()).rank <= (*fbn.as_ptr()).rank {
                            if ((*sfb.as_ptr()).first == from_b.inner
                                && (*sfb.as_ptr()).reverse
                                && (*sfb.as_ptr()).last == to_a.inner)
                                || ((*sfb.as_ptr()).first == to_a.inner
                                    && (*sfb.as_ptr()).last == from_b.inner)
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
                            (*sfa.as_ptr()).split(&tan);
                        }

                        if sfb == stb {
                            // split b
                            split = true;
                            (*sfb.as_ptr()).split(&tbn);
                        }

                        if split {
                            return self.flip(from_a, to_a, from_b, to_b);
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
                            reverse_segs(&sta, &sfb);
                        } else {
                            // Reverses the path (to_b, from_a).
                            reverse_segs(&stb, &sfa);
                        };
                    }
                    _ => panic!("DataNode without segment while flipping."),
                }
            },
            _ => panic!("Nullpointer"),
        }
    }

    #[inline]
    fn get(&self, index: usize) -> Option<TourNode> {
        self.nodes.get(index).copied()
    }

    #[inline]
    fn relation(&self, base: &TourNode, targ: &TourNode) -> NodeRel {
        match base.inner {
            Some(inner) => unsafe {
                match (*inner.as_ptr()).segment {
                    Some(seg) => {
                        match (
                            (*inner.as_ptr()).predecessor == targ.inner,
                            (*inner.as_ptr()).successor == targ.inner,
                            (*seg.as_ptr()).reverse ^ self.rev,
                        ) {
                            (true, false, true) | (false, true, false) => NodeRel::Predecessor,
                            (true, false, false) | (false, true, true) => NodeRel::Successor,
                            _ => NodeRel::None,
                        }
                    }
                    None => NodeRel::None,
                }
            },
            None => NodeRel::None,
        }
    }

    #[inline]
    fn successor(&self, node: &TourNode) -> Option<TourNode> {
        match node.inner {
            Some(inner) => unsafe {
                match (*inner.as_ptr()).segment {
                    Some(seg) => {
                        if (*seg.as_ptr()).reverse ^ self.rev {
                            (*inner.as_ptr()).predecessor.map(|_| TourNode {
                                inner: (*inner.as_ptr()).predecessor,
                            })
                        } else {
                            (*inner.as_ptr()).successor.map(|_| TourNode {
                                inner: (*inner.as_ptr()).successor,
                            })
                        }
                    }
                    None => None,
                }
            },
            None => None,
        }
    }

    #[inline]
    fn successor_at(&self, kin_index: usize) -> Option<TourNode> {
        match self.nodes.get(kin_index) {
            Some(kin) => self.successor(kin),
            None => None,
        }
    }

    #[inline]
    fn predecessor(&self, node: &TourNode) -> Option<TourNode> {
        match node.inner {
            Some(inner) => unsafe {
                match (*inner.as_ptr()).segment {
                    Some(seg) => {
                        if (*seg.as_ptr()).reverse ^ self.rev {
                            (*inner.as_ptr()).successor.map(|_| TourNode {
                                inner: (*inner.as_ptr()).successor,
                            })
                        } else {
                            (*inner.as_ptr()).predecessor.map(|_| TourNode {
                                inner: (*inner.as_ptr()).predecessor,
                            })
                        }
                    }
                    None => None,
                }
            },
            None => None,
        }
    }

    #[inline]
    fn predecessor_at(&self, kin_index: usize) -> Option<TourNode> {
        match self.nodes.get(kin_index) {
            Some(kin) => self.predecessor(kin),
            None => None,
        }
    }

    #[inline]
    fn rev(&mut self) {
        self.rev ^= true;
    }

    fn tour_order(&self) -> TourOrder {
        let mut result = Vec::with_capacity(self.nodes.len());
        let mut d = 0.;

        match self.nodes.first() {
            Some(first) => {
                result.push(first.index().get());
                let mut nopt = self.successor(first);

                loop {
                    match nopt {
                        Some(node) => {
                            d += self.distance(&self.predecessor(&node).unwrap(), &node);
                            if node.inner == first.inner {
                                break;
                            }

                            result.push(node.index().get());
                            nopt = self.successor(&node);
                        }
                        None => return TourOrder::default(),
                    }
                }

                TourOrder::with_cost(result, d)
            }
            None => TourOrder::default(),
        }
    }

    fn measure(&self, to: &TourOrder) -> Scalar {
        if self.len() == to.len() {
            let v = to.order();
            let mut cost = self.distance_at(v.first().unwrap(), v.last().unwrap());
            for pair in v.windows(2) {
                cost += self.distance_at(&pair[0], &pair[1]);
            }
            cost
        } else {
            0.
        }
    }

    fn reset(&mut self) {
        for node in &mut self.nodes {
            node.set_status(NodeStatus::Active);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[inline]
    fn total_distance(&self) -> crate::Scalar {
        self.total_dist
    }

    fn itr(&self) -> TourIter {
        TourIter {
            it: self.nodes.iter(),
        }
    }
}

// impl STree for TwoLevelList {
//     fn build_mst(&mut self) {
//         // A naive implementation of Prim's algorithm. Runtime is O(N^2).
//         // https://en.wikipedia.org/wiki/Prim%27s_algorithm
//         let n_nodes = self.nodes.len();
//         let mut selected = vec![false; n_nodes];
//         let mut processed = 0;

//         for nopt in &self.nodes {
//             match nopt.inner {
//                 Some(node) => unsafe {
//                     (*node.as_ptr()).mst_parent = None;
//                     (*node.as_ptr()).mst_final_edge = None;
//                 },
//                 None => panic!("Nullpointer"),
//             }
//         }

//         selected[0] = true;

//         while processed != n_nodes - 1 {
//             let (mut v_cand, mut w_cand, mut cost_cand) = (0, 0, Scalar::MAX);
//             for (v_idx, v_sel) in selected.iter().enumerate() {
//                 if *v_sel {
//                     for (w_idx, w_sel) in selected.iter().enumerate() {
//                         // The edge (v, w) is forbidden if its cost is equal to 0.
//                         let c = self.distance_at(v_idx, w_idx);
//                         if !*w_sel && c > 0. && c < cost_cand {
//                             v_cand = v_idx;
//                             w_cand = w_idx;
//                             cost_cand = c;
//                         }
//                     }
//                 }
//             }

//             let (vo, wo) = (self.nodes.get(v_cand), self.nodes.get(w_cand));
//             match (vo, wo) {
//                 (Some(v), Some(w)) => match w.inner {
//                     Some(vtx) => unsafe {
//                         (*vtx.as_ptr()).mst_parent = v.inner;
//                     },
//                     None => panic!("Nullpointer"),
//                 },
//                 _ => panic!("Nodes not found"),
//             }

//             selected[w_cand] = true;

//             processed += 1;
//         }
//     }

//     // Held-Karp lower bound
//     fn cost_m1t(&self) -> HeldKarpBound {
//         let (mut result, mut len_tree) = (0., 0.);
//         for nopt in &self.nodes {
//             match nopt.inner {
//                 Some(node) => unsafe {
//                     len_tree += (*node.as_ptr()).penalty_weight;
//                     (*node.as_ptr()).degree -= 2;

//                     match (*node.as_ptr()).mst_parent {
//                         Some(parent) => {
//                             (*node.as_ptr()).degree += 1;
//                             (*parent.as_ptr()).degree += 1;
//                             result += self.repo.distance(nopt.data(), &(*parent.as_ptr()).data);
//                         }
//                         None => panic!("Nullpointer in mst_parent"),
//                     }
//                 },
//                 None => panic!("Nullpointer"),
//             }
//         }

//         let mut cand_d = Scalar::MAX;
//         let mut cand = None;

//         for nopt in &self.nodes {
//             match nopt.inner {
//                 Some(node) => unsafe {
//                     if (*node.as_ptr()).degree == -1 {
//                         for other_opt in &self.nodes {
//                             match other_opt.inner {
//                                 Some(other) => {
//                                     if node == other
//                                         || (*node.as_ptr()).mst_parent == other_opt.inner
//                                         || (*other.as_ptr()).mst_parent == nopt.inner
//                                     {
//                                         continue;
//                                     }

//                                     let d = self.distance(&nopt, other_opt);
//                                     if d < cand_d {
//                                         cand_d = d;
//                                         (*node.as_ptr()).mst_final_edge = other_opt.inner;
//                                         cand = nopt.inner;
//                                     }
//                                 }
//                                 None => panic!("Nullpointer in mst_parent"),
//                             }
//                         }
//                     }
//                 },
//                 None => {
//                     panic!("Nullpointer")
//                 }
//             }
//         }

//         match cand {
//             Some(node) => unsafe {
//                 match (*node.as_ptr()).mst_parent {
//                     Some(parent) => {
//                         (*node.as_ptr()).degree += 1;
//                         (*parent.as_ptr()).degree += 1;
//                     }
//                     None => panic!("No mst parent"),
//                 }
//             },
//             None => panic!("Nullpointer"),
//         }

//         let mut total_deg = 0;
//         for nopt in &self.nodes {
//             match nopt.inner {
//                 Some(node) => unsafe {
//                     total_deg += (*node.as_ptr()).degree * (*node.as_ptr()).degree;
//                 },
//                 None => panic!("Nullpointer"),
//             }
//         }

//         if total_deg == 0 {
//             HeldKarpBound::Optimal
//         } else {
//             result += len_tree * 2. + cand_d;
//             HeldKarpBound::Value(result)
//         }
//     }
// }
