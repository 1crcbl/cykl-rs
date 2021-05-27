#![allow(unused_imports, dead_code)]

use crate::{
    tour::{
        tests::{create_repo, test_tour_order},
        Tour, TourImpltor, TourOrder, TwoLevelList,
    },
    MetricKind, Repo, RepoBuilder, Scalar,
};

use super::ropt::{move_2_opt, move_3_opt, Opt3Move};

#[test]
fn test_move_2_opt() {
    let tour = TwoLevelList::with_default_order(&create_repo(20), 20);
    let mut tour = TourImpltor::from(tour);
    let (node1, node3) = (tour.get(5).unwrap(), tour.get(10).unwrap());
    let (node2, node4) = (
        tour.successor(&node1).unwrap(),
        tour.successor(&node3).unwrap(),
    );

    move_2_opt(&mut tour, &node1, &node2, &node3, &node4);
    let mut expected: Vec<usize> = (0..6).collect();
    expected.append(&mut (6..11).rev().collect());
    expected.append(&mut (11..20).collect());
    let expected = TourOrder::with_ord(expected);

    test_tour_order(&tour, &expected);
}

#[test]
fn test_move_3_opt() {
    let tour = TwoLevelList::with_default_order(&create_repo(20), 20);
    let mut tour = TourImpltor::from(tour);
    let (f1, f2, f3) = (
        tour.get(5).unwrap(),
        tour.get(10).unwrap(),
        tour.get(15).unwrap(),
    );
    let (t1, t2, t3) = (
        tour.successor(&f1).unwrap(),
        tour.successor(&f2).unwrap(),
        tour.successor(&f3).unwrap(),
    );

    // Move 4
    move_3_opt(&mut tour, &f1, &t1, &f2, &t2, &f3, &t3, Opt3Move::Move4);
    let mut expected: Vec<usize> = (0..6).collect();
    expected.append(&mut (11..16).rev().collect());
    expected.append(&mut (6..11).collect());
    expected.append(&mut (16..20).collect());

    test_tour_order(&tour, &TourOrder::with_ord(expected));
}
