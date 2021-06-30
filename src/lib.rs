pub mod alg;

mod base;
pub use base::repo::DataNode;
pub use base::repo::NodeKind;
pub use base::repo::MatrixKind;
pub use base::repo::Repo;
pub use base::repo::RepoBuilder;

pub mod tour;

pub type Scalar = f64;

mod vrp;

#[cfg(test)]
mod tests {}
