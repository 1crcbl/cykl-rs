use crate::{
    alg::lkh::move_2_opt,
    tour::{is_excludable, NodeRel, Tour, TourNode, UpdateTourError},
    Scalar,
};

use super::types::SearchResult;

pub fn search_2_opt<T>(
    tour: &mut T,
    base: &TourNode,
    base_s: &TourNode,
) -> Result<SearchResult, UpdateTourError>
where
    T: Tour,
{
    let g0 = tour.distance(base, base_s);
    let mut g2_best = Scalar::MIN;
    let mut pair = None;

    for cand in base_s.candidates() {
        let g1 = g0 - tour.distance(base_s, cand);
        if tour.relation(base_s, cand) != NodeRel::None || g1 <= 0. {
            continue;
        }

        let cand_p = match tour.predecessor(cand) {
            Some(node) => node,
            None => Err(UpdateTourError::NodeNotFound)?,
        };

        // g2
        // let delta = tour.distance(&cand_p, cand) - tour.distance(base, &cand_p);
        let g2 = g1 + tour.distance(&cand_p, cand) - tour.distance(base, &cand_p);

        if g2 > 0. {
            // gain criterion satisfied.
            move_2_opt(tour, base, base_s, &cand_p, cand);
            return Ok(SearchResult::Gainful(g2));
        } else {
            // Non-gainful move.

            if g2 > g2_best && is_excludable(&cand_p, cand) {
                g2_best = g2;
                pair = Some((cand_p, *cand));
                // check if t3 and t4 can be excluded
            }
        }
    }

    if let Some((cand_1, cand_2)) = pair {
        move_2_opt(tour, base, base_s, &cand_1, &cand_2);
        return Ok(SearchResult::NonGainful(cand_1));
    }

    Err(UpdateTourError::SearchFailed)
}

pub fn search_3_opt<T>(
    _tour: &mut T,
    _head_1: &TourNode,
    _tail_1: &TourNode,
) -> Result<SearchResult, UpdateTourError>
where
    T: Tour,
{
    todo!()
}
