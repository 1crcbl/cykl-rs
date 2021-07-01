pub mod alg;

pub mod tour;

pub type Scalar = f64;

mod model;
pub use model::load_tsp;
pub use model::Model;
pub use model::RunConfig;
pub use model::RunConfigBuilder;

pub mod data;

mod tests;
