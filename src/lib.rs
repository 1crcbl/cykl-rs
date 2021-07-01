pub mod alg;

mod base;
pub use base::repo::DataNode;
pub use base::repo::MatrixKind;
pub use base::repo::NodeKind;
pub use base::repo::Repo;
pub use base::repo::RepoBuilder;

pub mod tour;

pub type Scalar = f64;

mod model;
pub use model::load_tsp;
pub use model::Model;

pub mod data;

mod tests;
