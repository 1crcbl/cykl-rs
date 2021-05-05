use super::Solver;

use crate::{
    tour::{Tour, TourOrder, Vertex},
    Scalar,
};

use rand::prelude::*;

pub struct Greedy {}

impl Greedy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Solver for Greedy {
    fn solve<T>(&self, tour: &mut T) -> TourOrder
    where
        T: Tour,
    {
        tour.reset();

        let size = tour.size();
        let mut result = Vec::with_capacity(tour.size());
        let mut total_dist = 0.;

        let mut rng = rand::thread_rng();
        let mut node_idx = rng.gen_range(0..size);
        let beg_idx = node_idx;

        while result.len() != size {
            let mut cnd = None;
            let mut dist = Scalar::MAX;

            for kin_idx in 0..size {
                let kin = tour.get(kin_idx).unwrap();
                if kin.is_visited() || kin_idx == node_idx {
                    continue;
                }

                let new_dist = tour.distance_at(node_idx, kin_idx);

                if new_dist < dist {
                    dist = new_dist;
                    cnd = Some((kin_idx, dist));
                }
            }

            result.push(node_idx);
            tour.visited_at(node_idx, true);

            match cnd {
                Some((cnd_idx, cnd_dist)) => {
                    node_idx = cnd_idx;
                    total_dist += cnd_dist;
                }
                None => {
                    total_dist += tour.distance_at(node_idx, beg_idx);
                }
            };
        }

        TourOrder::with_dist(result, total_dist)
    }
}

#[allow(dead_code, unused_imports)]
mod tests {
    use crate::{metric::MetricKind, node::Container, tour::Array};

    use super::super::tests::load_burma_test;
    use super::*;

    #[test]
    fn test_greedy() {
        let container = load_burma_test();
        let nng = Greedy::new();
        let mut tour = Array::new(&container);
        let result = nng.solve(&mut tour);
        assert_eq!(container.size(), result.order().len());
    }
}
