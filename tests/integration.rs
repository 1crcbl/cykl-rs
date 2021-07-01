#![cfg(test)]
use cykl::load_tsp;

#[test]
fn test_eil22() {
    let mut model = load_tsp("./tests/data/eil22.vrp", 100);
    assert_eq!(1, model.n_depots());
    assert_eq!(22, model.n_nodes());

    model.solve();
}
