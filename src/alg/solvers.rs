use crate::{
    data::GetIndex,
    tour::{NodeStatus, Tour, TourOrder, UpdateTourError},
    Scalar,
};

/// Uses greedy algorithm to construct a tour.
pub fn solve_greedy<T>(
    tour: &mut T,
    starters: &[usize],
) -> Result<Option<TourOrder>, UpdateTourError>
where
    T: Tour,
{
    if tour.len() == 0 {
        return Ok(None);
    }

    let len = tour.len();
    let mut best_tour = None;
    let mut best_cost = Scalar::MAX;

    for starter in starters {
        tour.reset();

        let mut v = Vec::with_capacity(tour.len());
        let mut node = match tour.get(*starter) {
            Some(node) => node,
            None => return Err(UpdateTourError::NodeNotFound),
        };

        v.push(node.index().get());
        node.set_status(NodeStatus::Fixed);

        while v.len() != len {
            let mut chosen = None;
            for cand in node.candidates() {
                if !cand.is_status(NodeStatus::Active) {
                    continue;
                }

                chosen = Some(*cand);
                break;
            }

            let mut next = chosen.unwrap_or_else(|| {
                let mut d = Scalar::MAX;
                let mut cand = None;

                for next_node in tour.itr() {
                    if !next_node.is_status(NodeStatus::Active) {
                        continue;
                    }

                    let next_d = tour.distance(&node, &next_node);
                    if next_d < d && next_d > 0. {
                        d = next_d;
                        cand = Some(next_node);
                    }
                }

                match cand {
                    Some(next_node) => next_node,
                    None => panic!("No node found"),
                }
            });

            next.set_status(NodeStatus::Fixed);
            v.push(next.index().get());
            node = next;
        }

        let mut to = TourOrder::with_ord(v);
        let cost = tour.measure(&to);

        if cost < best_cost {
            to.set_cost(cost);
            best_tour = Some(to);
            best_cost = cost;
        }
    }

    if let Some(to) = &best_tour {
        tour.apply(to)?;
    }

    Ok(best_tour)
}
