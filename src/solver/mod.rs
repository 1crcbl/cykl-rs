use crate::tour::{Tour, TourOrder};

mod greedy;
pub use greedy::Greedy as Greedy;

pub trait Solver {
    fn solve<T>(&self, tour: &mut T) -> TourOrder
    where
        T: Tour;
}

#[allow(dead_code, unused_imports)]
mod tests {
    use crate::{metric::MetricKind, node::Container};

    pub fn load_burma_test() -> Container {
        let mut container = Container::new(MetricKind::Geo);
        container.add(16.47, 96.10, 0.);
        container.add(16.47, 94.44, 0.);
        container.add(20.09, 92.54, 0.);
        container.add(22.39, 93.37, 0.);
        container.add(25.23, 97.24, 0.);
        container.add(22.00, 96.05, 0.);
        container.add(20.47, 97.02, 0.);
        container.add(17.20, 96.29, 0.);
        container.add(16.30, 97.38, 0.);
        container.add(14.05, 98.12, 0.);
        container.add(16.53, 97.38, 0.);
        container.add(21.52, 95.59, 0.);
        container.add(19.41, 97.13, 0.);
        container.add(20.09, 94.55, 0.);
        container
    }
}