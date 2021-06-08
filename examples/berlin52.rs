use std::path::Path;

use cykl::{
    alg::lkh::{move_3_opt, Opt3Move},
    tour::{Tour, TwoLevelList},
    Repo,
};
use tspf::TspBuilder;

/// In this example, we construct a tour with two-level list from a file containing
/// a list of node coordinates. Then we will manipulate the tour by applying a 3-opt move.
pub fn main() {
    let tsp = TspBuilder::parse_path(Path::new("./examples/berlin52.tsp")).unwrap();
    let repo = Repo::from(&tsp);
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
