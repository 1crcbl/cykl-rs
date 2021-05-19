use std::{collections::VecDeque, iter::FromIterator};

use crate::{
    tour::{Tour, TourIter, TourOrder, TwoLevelList, Vertex},
    Scalar,
};

/// Constructs a tour by using the nearest-neigbour method coupled with the breadth-first search.
pub fn nn_simple(tour: &mut TwoLevelList) -> TourOrder {
    let len = tour.len();
    let repo = tour.repo();
    let mut ord = TourOrder::with_capacity(tour.len());

    for base in tour.itr_mut() {
        if base.degree() == 2 {
            continue;
        }

        let mut deque = VecDeque::from_iter(base.cands_itr());
        let mut deque2 = Vec::new();

        let mut cand = None;
        let mut cand_d = Scalar::MAX;

        loop {
            while let Some(targ) = deque.pop_front() {
                let d = repo.distance(base.data(), targ.data());
                if d < cand_d {
                    cand_d = d;
                    cand = Some(targ);
                }
                deque2.push(targ);
            }

            match cand {
                Some(targ) => {
                    ord.add(targ.index());
                    base.add_degree();
                    break;
                }
                None => {
                    deque2.iter().for_each(|t| {
                        deque.extend(t.cands_itr());
                    });
                }
            }
        }

        if ord.len() == len {
            break;
        }
    }

    ord
}
