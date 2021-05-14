use crate::{MetricKind, Repo, RepoBuilder};

#[allow(dead_code)]
fn load_burma_test() -> Repo {
    let builder = RepoBuilder::new(MetricKind::Geo).capacity(14);
    let mut repo = builder.build();
    repo.add(16.47, 96.10, 0.);
    repo.add(16.47, 94.44, 0.);
    repo.add(20.09, 92.54, 0.);
    repo.add(22.39, 93.37, 0.);
    repo.add(25.23, 97.24, 0.);
    repo.add(22.00, 96.05, 0.);
    repo.add(20.47, 97.02, 0.);
    repo.add(17.20, 96.29, 0.);
    repo.add(16.30, 97.38, 0.);
    repo.add(14.05, 98.12, 0.);
    repo.add(16.53, 97.38, 0.);
    repo.add(21.52, 95.59, 0.);
    repo.add(19.41, 97.13, 0.);
    repo.add(20.09, 94.55, 0.);
    repo
}

#[allow(dead_code, unused_imports)]
mod tests_greedy {
    use crate::{
        solver::{Greedy, Solver},
        tour::Array,
    };

    use super::*;

    #[test]
    fn test_greedy_array() {
        let container = load_burma_test();
        let nng = Greedy::new();
        let mut tour = Array::new(&container);
        let result = nng.solve(&mut tour);
        assert_eq!(container.size(), result.order().len());
    }
}
