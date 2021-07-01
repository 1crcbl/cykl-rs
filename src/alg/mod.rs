mod tour_gen;

pub mod lkh;

mod cand_gen;
pub use cand_gen::cand_gen_nn;

pub mod solvers;

mod tests;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SolverKind {
    Greedy(Vec<usize>),
}
