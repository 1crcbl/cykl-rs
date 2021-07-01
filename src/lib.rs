pub mod alg;

pub mod tour;

pub type Scalar = f64;

mod model;
pub use model::load_tsp;
pub use model::Model;

pub mod data;

mod tests;
