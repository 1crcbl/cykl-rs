#![cfg(test)]
use crate::{
    combine_range,
    tour::{
        tests::{create_store, test_tour_order},
        Tour, TourOrder, TwoLevelList,
    },
    tour_order,
};

use super::lkh::{move_2_opt, move_3_opt, move_4_opt, Opt3Move, Opt4SeqMove};

#[test]
fn test_move_2_opt() {
    let mut tour = TwoLevelList::new(create_store(20), 20);
    let (node1, node3) = (tour.get(5).unwrap(), tour.get(10).unwrap());
    let (node2, node4) = (
        tour.successor(&node1).unwrap(),
        tour.successor(&node3).unwrap(),
    );

    move_2_opt(&mut tour, &node1, &node2, &node3, &node4);
    let expected = tour_order!(0..6, (6..11).rev(), 11..20);

    test_tour_order(&tour, &expected);
}

#[test]
fn test_move_3_opt() {
    let mut tour = TwoLevelList::new(create_store(20), 20);
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
    move_3_opt(
        &mut tour,
        (&f1, &t1),
        (&f2, &t2),
        (&f3, &t3),
        Opt3Move::Move4,
    );
    test_tour_order(&tour, &tour_order![(0..6), (11..16).rev(), 6..11, 16..20]);

    // Move 7
    assert!(tour.apply(&nat_ord).is_ok());
    move_3_opt(
        &mut tour,
        (&f1, &t1),
        (&f2, &t2),
        (&f3, &t3),
        Opt3Move::Move7,
    );
    test_tour_order(&tour, &tour_order![0..6, 11..16, 6..11, 16..20]);
}

#[allow(unused_macros)]
macro_rules! tour_4 {
    ($($x:expr),+) => {{
        let mut a: Vec<usize> = (0..=5).collect();
        a.append(&mut combine_range!($($x),*));
        a.append(&mut (21..30).collect());
        TourOrder::with_ord(a)
    }};
}

#[test]
fn test_move_4_opt() {
    let mut tour = TwoLevelList::new(create_store(30), 30);
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

    let mut fn4 = |mv: Opt4SeqMove, exp: TourOrder| {
        assert!(tour.apply(&nat_ord).is_ok());
        move_4_opt(
            &mut tour,
            (&f1, &t1),
            (&f2, &t2),
            (&f3, &t3),
            (&f4, &t4),
            mv,
        );
        test_tour_order(&tour, &exp);
    };

    fn4(
        Opt4SeqMove::Move1,
        tour_4![(6..11).rev(), (11..16).rev(), (16..21).rev()],
    );

    fn4(Opt4SeqMove::Move2, tour_4![11..16, 6..11, (16..21).rev()]);

    fn4(
        Opt4SeqMove::Move3,
        tour_4![11..16, (6..11).rev(), (16..21).rev()],
    );

    fn4(
        Opt4SeqMove::Move4,
        tour_4![(11..=15).rev(), 6..=10, (16..=20).rev()],
    );

    fn4(
        Opt4SeqMove::Move5,
        tour_4![16..=20, 6..=10, (11..=15).rev()],
    );

    fn4(
        Opt4SeqMove::Move6,
        tour_4![16..=20, (6..=10).rev(), 11..=15],
    );

    fn4(
        Opt4SeqMove::Move7,
        tour_4![(16..=20).rev(), 6..=10, (11..=15).rev()],
    );

    fn4(
        Opt4SeqMove::Move8,
        tour_4![(16..=20).rev(), (6..=10).rev(), 11..=15],
    );

    fn4(
        Opt4SeqMove::Move9,
        tour_4![(16..=20).rev(), (6..=10).rev(), (11..=15).rev()],
    );

    fn4(
        Opt4SeqMove::Move10,
        tour_4![(6..=10).rev(), 16..=20, 11..=15],
    );

    fn4(
        Opt4SeqMove::Move11,
        tour_4![(6..=10).rev(), 16..=20, (11..=15).rev()],
    );

    fn4(
        Opt4SeqMove::Move12,
        tour_4![(6..=10).rev(), (16..=20).rev(), 11..=15],
    );

    fn4(
        Opt4SeqMove::Move13,
        tour_4![11..=15, (16..=20).rev(), 6..=10],
    );

    fn4(
        Opt4SeqMove::Move14,
        tour_4![11..=15, (16..=20).rev(), (6..=10).rev()],
    );

    fn4(
        Opt4SeqMove::Move15,
        tour_4![(11..=15).rev(), 16..=20, 6..=10],
    );

    fn4(
        Opt4SeqMove::Move16,
        tour_4![(11..=15).rev(), 16..=20, (6..=10).rev()],
    );

    fn4(
        Opt4SeqMove::Move17,
        tour_4![(11..=15).rev(), (16..=20).rev(), (6..=10).rev()],
    );

    fn4(
        Opt4SeqMove::Move18,
        tour_4![16..=20, 11..=15, (6..=10).rev()],
    );

    fn4(
        Opt4SeqMove::Move19,
        tour_4![16..=20, (11..=15).rev(), 6..=10],
    );

    fn4(
        Opt4SeqMove::Move20,
        tour_4![(16..=20).rev(), 11..=15, 6..=10],
    );
}

// fn create_repo_2() -> Repo {
//     let mut repo = RepoBuilder::new(tspf::WeightKind::Euc2d)
//         .capacity(16)
//         .build();
//     repo.add(NodeKind::Customer, vec![38.24, 20.42, 0.]);
//     repo.add(NodeKind::Customer, vec![39.57, 26.15, 0.]);
//     repo.add(NodeKind::Customer, vec![40.56, 25.32, 0.]);
//     repo.add(NodeKind::Customer, vec![36.26, 23.12, 0.]);
//     repo.add(NodeKind::Customer, vec![33.48, 10.54, 0.]);
//     repo.add(NodeKind::Customer, vec![37.56, 12.19, 0.]);
//     repo.add(NodeKind::Customer, vec![38.42, 13.11, 0.]);
//     repo.add(NodeKind::Customer, vec![37.52, 20.44, 0.]);
//     repo.add(NodeKind::Customer, vec![41.23, 9.10, 0.]);
//     repo.add(NodeKind::Customer, vec![41.17, 13.05, 0.]);
//     repo.add(NodeKind::Customer, vec![36.08, -5.21, 0.]);
//     repo.add(NodeKind::Customer, vec![38.47, 15.13, 0.]);
//     repo.add(NodeKind::Customer, vec![38.15, 15.35, 0.]);
//     repo.add(NodeKind::Customer, vec![37.51, 15.17, 0.]);
//     repo.add(NodeKind::Customer, vec![35.49, 14.32, 0.]);
//     repo.add(NodeKind::Customer, vec![39.36, 19.56, 0.]);
//     repo
// }

// #[test]
// fn test_cand_gen_nn() {
//     let mut tour = TwoLevelList::new(create_repo_2(), 4);
//     cand_gen_nn(&mut tour, 5);

//     for node in tour.itr() {
//         assert_eq!(5, node.candidates().len());
//     }
// }
