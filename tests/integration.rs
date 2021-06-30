#![cfg(test)]
use cykl::{Repo, alg::lkh::{Opt3Move, move_3_opt}, tour::{Tour, TwoLevelList}};
use std::path::Path;
use tspf::TspBuilder;

#[test]
fn test_eil22() {
    let tsp = TspBuilder::parse_path(Path::new("./tests/data/eil22.vrp")).unwrap();
    assert_eq!(1, tsp.depots().len());
    let _repo = Repo::from(tsp);
    // let mut tll = TwoLevelList::new(&repo, 13);

    // let (node1, node2, node3) = (
    //     tll.get(5).unwrap(),
    //     tll.get(20).unwrap(),
    //     tll.get(40).unwrap(),
    // );
    // let (node1_nxt, node2_nxt, node3_nxt) = (
    //     tll.successor(&node1).unwrap(),
    //     tll.successor(&node2).unwrap(),
    //     tll.successor(&node3).unwrap(),
    // );

    // move_3_opt(
    //     &mut tll,
    //     &node1,
    //     &node1_nxt,
    //     &node2,
    //     &node2_nxt,
    //     &node3,
    //     &node3_nxt,
    //     Opt3Move::Move1,
    // );

    // // Get the node sequence after the 3-opt operation.
    // let _ = tll.tour_order();
}

#[test]
fn test_berlin52() {
    let tsp = TspBuilder::parse_path(Path::new("./tests/data/berlin52.tsp")).unwrap();
    let repo = Repo::from(tsp);
    let mut tll = TwoLevelList::new(&repo, 13);

    let (node1, node2, node3) = (
        tll.get(5).unwrap(),
        tll.get(20).unwrap(),
        tll.get(40).unwrap(),
    );
    let (node1_nxt, node2_nxt, node3_nxt) = (
        tll.successor(&node1).unwrap(),
        tll.successor(&node2).unwrap(),
        tll.successor(&node3).unwrap(),
    );

    move_3_opt(
        &mut tll,
        &node1,
        &node1_nxt,
        &node2,
        &node2_nxt,
        &node3,
        &node3_nxt,
        Opt3Move::Move1,
    );

    // Get the node sequence after the 3-opt operation.
    let _ = tll.tour_order();
}