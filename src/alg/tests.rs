#![allow(unused_imports, dead_code)]

use crate::{
    combine_range,
    tour::{
        tests::{create_repo, test_tour_order},
        Tour, TourImpltor, TourOrder, TwoLevelList,
    },
    MetricKind, Repo, RepoBuilder, Scalar,
};

use super::ropt::{move_2_opt, move_3_opt, move_4_opt, Move4Opt, Opt3Move};

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

    let nat_ord = TourOrder::with_nat_ord(20);

    // Move 4
    move_3_opt(&mut tour, &f1, &t1, &f2, &t2, &f3, &t3, Opt3Move::Move4);
    test_tour_order(
        &tour,
        &TourOrder::with_ord(combine_range![(0..6), (11..16).rev(), 6..11, 16..20]),
    );

    // Move 7
    tour.apply(&nat_ord);
    move_3_opt(&mut tour, &f1, &t1, &f2, &t2, &f3, &t3, Opt3Move::Move7);
    test_tour_order(
        &tour,
        &TourOrder::with_ord(combine_range![0..6, 11..16, 6..11, 16..20]),
    );
}

#[test]
fn test_move_4_opt() {
    let tour = TwoLevelList::with_default_order(&create_repo(30), 30);
    let mut tour = TourImpltor::from(tour);
    let (f1, f2, f3, f4) = (
        tour.get(5).unwrap(),
        tour.get(10).unwrap(),
        tour.get(15).unwrap(),
        tour.get(20).unwrap(),
    );
    let (t1, t2, t3, t4) = (
        tour.successor(&f1).unwrap(),
        tour.successor(&f2).unwrap(),
        tour.successor(&f3).unwrap(),
        tour.successor(&f4).unwrap(),
    );

    let nat_ord = TourOrder::with_nat_ord(30);

    // SeqMove1
    move_4_opt(
        &mut tour,
        &f1,
        &t1,
        &f2,
        &t2,
        &f3,
        &t3,
        &f4,
        &t4,
        Move4Opt::SeqMove1,
    );
    test_tour_order(
        &tour,
        &TourOrder::with_ord(combine_range![
            0..6,
            (6..11).rev(),
            (11..16).rev(),
            (16..21).rev(),
            21..30
        ]),
    );

    // SeqMove2
    tour.apply(&nat_ord);
    move_4_opt(
        &mut tour,
        &f1,
        &t1,
        &f2,
        &t2,
        &f3,
        &t3,
        &f4,
        &t4,
        Move4Opt::SeqMove2,
    );
    test_tour_order(
        &tour,
        &TourOrder::with_ord(combine_range![0..6, 11..16, 6..11, (16..21).rev(), 21..30]),
    );
}
