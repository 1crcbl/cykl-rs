use crate::{
    tour::{NodeStatus, Tour, UpdateTourError},
    Scalar,
};

/// Uses greedy algorithm to construct a tour.
pub fn solve_greedy<T>(tour: &mut T, starter: Option<usize>) -> Result<(), UpdateTourError>
where
    T: Tour,
{
    if tour.len() == 0 {
        return Ok(());
    }

    let len = tour.len();
    let mut v = Vec::with_capacity(tour.len());
    let mut node = match tour.get(starter.unwrap_or(0)) {
        Some(node) => node,
        None => return Err(UpdateTourError::NodeNotFound),
    };

    v.push(node.index());
    node.set_status(NodeStatus::Anchored);

    while v.len() != len {
        let mut chosen = None;
        for cand in node.candidates() {
            if !cand.is_status(NodeStatus::Active) {
                continue;
            }

            chosen = Some(*cand);
            break;
        }

        let mut next = match chosen {
            Some(next_node) => next_node,
            None => {
                let mut d = Scalar::MAX;
                let mut cand = None;

                for next_node in tour.itr() {
                    if next_node.is_status(NodeStatus::Active) {
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
                    None => panic!("Something wrong"),
                }
            }
        };

        next.set_status(NodeStatus::Anchored);
        v.push(next.index());
        node = next;
    }

    todo!()

    // tour.apply(&TourOrder::with_ord(v))
}
