use crate::{tour::TourNode, Scalar};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KOpt {
    /// Corresponds to the 2-opt case.
    Opt2,
    /// Corresponds to the 3-opt case.
    Opt3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchResult {
    Gainful(Scalar),
    NonGainful(TourNode),
}
