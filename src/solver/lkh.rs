use crate::tour::HeldKarpBound;
use crate::{
    tour::{STree, Tour, TourOrder},
    Scalar,
};
use HeldKarpBound::*;

pub fn lkh_solver<T>(tour: &mut T) -> TourOrder
where
    T: Tour + STree,
{
    let n_runs = 100;

    tour.reset();
    cand_set(tour);

    let mut best_cost = Scalar::MAX;

    for _ in 0..n_runs {
        let cost = search(tour);
        if cost < best_cost {
            // save current best tour?
            best_cost = cost;
        }
    }

    // return best_tour
    todo!("impl lkh");
}

// Create candidate set for a tour.
fn cand_set<T>(tour: &mut T)
where
    T: Tour + STree,
{
    let _ = ascent(tour);
    // max_alpha = excess * fabs(lower_bound) ?;
    // generate candidates
}

fn ascent<T>(tour: &mut T)
where
    T: Tour + STree,
{
    tour.build_mst();
    let hkb = tour.cost_m1t();

    let _ = match hkb {
        Value(w) => w,
        Optimal => {
            todo!()
        }
    };
}

fn search<T>(_tour: &mut T) -> Scalar
where
    T: Tour,
{
    todo!()
}
