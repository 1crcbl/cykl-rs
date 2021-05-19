use crate::tour::{Tour, TourOrder};

mod greedy;
pub use greedy::Greedy;

pub mod lkh;

pub mod nn;

mod tests;

pub trait Solver {
    fn solve<T>(&self, tour: &mut T) -> TourOrder
    where
        T: Tour;
}
