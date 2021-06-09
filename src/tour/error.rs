#[derive(Debug)]
pub enum UpdateTourError {
    BrokenTour,

    TourLenMismatched { expected: usize, received: usize },

    InvalidTourOrder,

    NodeNotFound,

    SearchFailed,
}
