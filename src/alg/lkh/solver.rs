use std::collections::VecDeque;

use crate::{
    alg::tour_gen::init_tour,
    tour::{NodeStatus, Tour, TourNode, TourOrder, UpdateTourError},
    Scalar,
};

use super::{searches::search_2_opt, types::SearchResult, KOpt};

pub fn solve_lkh<T>(tour: &mut T, kopt: KOpt, trials: usize) -> Result<(), UpdateTourError>
where
    T: Tour,
{
    let len = tour.len();
    let mut best_order = TourOrder::default();

    for _ in 0..trials {
        init_tour(tour)?;

        let original_order = tour.tour_order();
        let mut current_cost = original_order.cost();

        let mut active = VecDeque::with_capacity(len);

        for mut node in tour.itr() {
            if node.is_best_neighbours(&tour.successor(&node).unwrap(), 0)
                || node.is_best_neighbours(&tour.predecessor(&node).unwrap(), 0)
            {
                node.set_status(NodeStatus::Anchored);
            } else {
                node.set_status(NodeStatus::Active);
                active.push_back(node)
            }
        }

        while let Some(mut base) = active.pop_front() {
            base.set_status(NodeStatus::Anchored);

            let successor = match tour.successor(&base) {
                Some(s) => s,
                None => return Err(UpdateTourError::NodeNotFound),
            };

            let predecessor = match tour.predecessor(&base) {
                Some(p) => p,
                None => return Err(UpdateTourError::NodeNotFound),
            };

            let mut flag = true;
            for ii in 0..2 {
                let gain = if ii == 1 {
                    search(tour, kopt, &base, &successor)?
                } else {
                    match tour.relation(&base, &predecessor) {
                        crate::tour::NodeRel::Predecessor => tour.rev(),
                        crate::tour::NodeRel::Successor => {}
                        crate::tour::NodeRel::None => return Err(UpdateTourError::BrokenTour),
                    };

                    search(tour, kopt, &base, &predecessor)?
                };

                if gain > 0. {
                    current_cost -= gain;
                    break;
                } else {
                    base.set_status(NodeStatus::Active);

                    if flag {
                        active.push_back(base);
                        flag = false;
                    }
                }
            }
        }

        if current_cost < best_order.cost() {
            best_order = tour.tour_order();
        }
    }

    Ok(())
}

fn search<T>(
    tour: &mut T,
    kopt: KOpt,
    base: &TourNode,
    base_s: &TourNode,
) -> Result<Scalar, UpdateTourError>
where
    T: Tour,
{
    let mut next = Some(*base_s);

    while let Some(targ) = next {
        let result = match kopt {
            KOpt::Opt2 => search_2_opt(tour, base, &targ)?,
            KOpt::Opt3 => todo!(),
        };

        match result {
            SearchResult::Gainful(gain) => {
                return Ok(gain);
            }
            SearchResult::NonGainful(node) => {
                next = Some(node);
            }
        };
    }

    Ok(0.)
}
