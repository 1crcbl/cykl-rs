#![cfg(test)]
use cykl::load_tsp;

#[test]
fn test_a280() {
    let model = load_tsp("./tests/data/a280.tsp", 20);
    assert_eq!(0, model.n_depots());
    assert_eq!(280, model.n_nodes());
}
