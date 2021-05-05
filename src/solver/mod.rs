use crate::tour::{Tour, TourOrder};

mod greedy;
pub use greedy::Greedy;

mod tests;

pub trait Solver {
    fn solve<T>(&self, tour: &mut T) -> TourOrder
    where
        T: Tour;
}
