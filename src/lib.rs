mod base;
pub use base::repo::Repo;
pub use base::repo::DataNode;
pub use base::repo::MetricKind;

pub mod solver;
pub mod tour;

pub type Scalar = f64;

#[cfg(test)]
mod tests {}
