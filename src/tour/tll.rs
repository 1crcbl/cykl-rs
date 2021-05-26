use std::ptr::NonNull;

use crate::{
    tour::{
        node::{reverse_int_seg, reverse_segs},
        HeldKarpBound,
    },
    Repo, Scalar,
};

use super::{
    between,
    node::{to_nonnull, Segment},
    STree, Tour, TourIter, TourNode, TourOrder,
};

#[derive(Debug)]
pub struct TwoLevelList {
    repo: Repo,
    pub(crate) segments: Vec<Option<NonNull<Segment>>>,
    nodes: Vec<TourNode>,
    total_dist: Scalar,
}

impl TwoLevelList {
    pub fn new(repo: &Repo, max_grouplen: usize) -> Self {
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

        let nodes = repo.into_iter().map(|node| TourNode::new(node)).collect();

        Self {
            repo: repo.clone(),
            nodes: nodes,
            segments: segments,
            total_dist: 0.,
        }
    }

    pub fn with_default_order(repo: &Repo, max_grouplen: usize) -> Self {
        let mut result = Self::new(repo, max_grouplen);
        result.apply(&TourOrder::with_ord((0..repo.size()).collect()));
        result
    }

    pub fn repo(&self) -> Repo {
        self.repo.clone()
    }
}

impl Tour for TwoLevelList {
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

                        match (el_v.inner, el_next.inner, el_prev.inner) {
                            (Some(vtx), Some(vtx_nxt), Some(vtx_prv)) => {
                                (*vtx.as_ptr()).predecessor = el_prev.inner;
                                (*vtx.as_ptr()).successor = el_next.inner;
                                (*vtx.as_ptr()).rank = (iv - beg_seg) as i32;
                                (*vtx.as_ptr()).segment = *els;

                                (*vtx_nxt.as_ptr()).predecessor = el_v.inner;
                                (*vtx_prv.as_ptr()).successor = el_v.inner;

                                self.total_dist += self
                                    .repo
                                    .distance(&(*vtx.as_ptr()).data, &(*vtx_nxt.as_ptr()).data);
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
                self.repo
                    .distance(&(*ai.as_ptr()).data, &(*bi.as_ptr()).data)
            },
            _ => 0.,
        }
    }

    #[inline]
    fn distance_at(&self, a: usize, b: usize) -> crate::Scalar {
        self.repo.distance_at(a, b)
    }

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
    }

    #[inline]
    fn get(&self, index: usize) -> Option<TourNode> {
        match self.nodes.get(index) {
            Some(v) => Some(TourNode { inner: v.inner }),
            None => None,
        }
    }

    #[inline]
    fn successor(&self, node: &TourNode) -> Option<TourNode> {
        match node.inner {
            Some(inner) => unsafe {
                match (*inner.as_ptr()).segment {
                    Some(seg) => {
                        if (*seg.as_ptr()).reverse {
                            match (*inner.as_ptr()).predecessor {
                                Some(_) => Some(TourNode {
                                    inner: (*inner.as_ptr()).predecessor,
                                }),
                                None => None,
                            }
                        } else {
                            match (*inner.as_ptr()).successor {
                                Some(_) => Some(TourNode {
                                    inner: (*inner.as_ptr()).successor,
                                }),
                                None => None,
                            }
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
                        if (*seg.as_ptr()).reverse {
                            match (*inner.as_ptr()).successor {
                                Some(_) => Some(TourNode {
                                    inner: (*inner.as_ptr()).successor,
                                }),
                                None => None,
                            }
                        } else {
                            match (*inner.as_ptr()).predecessor {
                                Some(_) => Some(TourNode {
                                    inner: (*inner.as_ptr()).predecessor,
                                }),
                                None => None,
                            }
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

    // TODO: require reimpl
    fn gen_cands(&mut self, k: usize) {
        let len = self.nodes.len();
        for (base_idx, base) in self.nodes.iter().enumerate() {
            let mut targ_idx = (base_idx + 1) % len;
            let mut cands = vec![None; k];
            let mut cands_d = vec![Scalar::MAX; k];
            let mut count = 0;

            while targ_idx != base_idx {
                let targ = &self.nodes[targ_idx];

                match (base.inner, targ.inner) {
                    (Some(_nb), Some(_nt)) => {
                        if count < k {
                            count += 1;
                        }
                        let mut c_idx = count - 1;

                        let d = self.distance(base, targ);

                        while c_idx > 0 && d < cands_d[c_idx - 1] {
                            cands[c_idx] = cands[c_idx - 1];
                            cands_d[c_idx] = cands_d[c_idx - 1];
                            c_idx -= 1;
                        }

                        if d < cands_d[c_idx] {
                            cands[c_idx] = targ.inner;
                            cands_d[c_idx] = d;
                        }
                    }
                    _ => panic!("Nullpointers"),
                }

                targ_idx = (targ_idx + 1) % len;
            }

            unsafe {
                // TODO: remove unwrap
                (*base.inner.unwrap().as_ptr()).cands = cands;
            }
        }
    }

    fn itr(&self) -> TourIter {
        TourIter {
            it: self.nodes.iter(),
        }
    }
}

impl STree for TwoLevelList {
    fn build_mst(&mut self) {
        // A naive implementation of Prim's algorithm. Runtime is O(N^2).
        // https://en.wikipedia.org/wiki/Prim%27s_algorithm
        let n_nodes = self.nodes.len();
        let mut selected = vec![false; n_nodes];
        let mut processed = 0;

        for nopt in &self.nodes {
            match nopt.inner {
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
                (Some(v), Some(w)) => match w.inner {
                    Some(vtx) => unsafe {
                        (*vtx.as_ptr()).mst_parent = v.inner;
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
            match nopt.inner {
                Some(node) => unsafe {
                    len_tree += (*node.as_ptr()).penalty_weight;
                    (*node.as_ptr()).degree -= 2;

                    match (*node.as_ptr()).mst_parent {
                        Some(parent) => {
                            (*node.as_ptr()).degree += 1;
                            (*parent.as_ptr()).degree += 1;
                            result += self.repo.distance(nopt.data(), &(*parent.as_ptr()).data);
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
            match nopt.inner {
                Some(node) => unsafe {
                    if (*node.as_ptr()).degree == -1 {
                        for other_opt in &self.nodes {
                            match other_opt.inner {
                                Some(other) => {
                                    if node == other
                                        || (*node.as_ptr()).mst_parent == other_opt.inner
                                        || (*other.as_ptr()).mst_parent == nopt.inner
                                    {
                                        continue;
                                    }

                                    let d = self.distance(&nopt, other_opt);
                                    if d < cand_d {
                                        cand_d = d;
                                        (*node.as_ptr()).mst_final_edge = other_opt.inner;
                                        cand = nopt.inner;
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
            match nopt.inner {
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
