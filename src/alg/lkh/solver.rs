use std::collections::VecDeque;

use crate::{
    alg::tour_gen::init_tour,
    tour::{NodeStatus, Tour, UpdateTourError},
};

pub fn solve_lkh<T>(tour: &mut T, trials: usize) -> Result<(), UpdateTourError>
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

        while let Some(_) = active.pop_front() {}
    }

    Ok(())
}
