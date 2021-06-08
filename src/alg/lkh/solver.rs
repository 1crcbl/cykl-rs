use std::collections::VecDeque;

use crate::{
    alg::tour_gen::init_tour,
    tour::{NodeStatus, Tour, UpdateTourError},
};

use super::{searches::search, KOpt};

pub fn solve_lkh<T>(tour: &mut T, kopt: KOpt, trials: usize) -> Result<(), UpdateTourError>
where
    T: Tour,
{
    let len = tour.len();

    for _ in 0..trials {
        init_tour(tour)?;

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

        while let Some(mut node) = active.pop_front() {
            node.set_status(NodeStatus::Anchored);
            // TODO: use NoneError once the feature is stablised.
            // https://doc.rust-lang.org/std/option/struct.NoneError.html
            let successor = match tour.successor(&node) {
                Some(node) => node,
                None => Err(UpdateTourError::NodeNotFound)?,
            };
            search(tour, kopt, &node, &successor);
        }
    }

    Ok(())
}
