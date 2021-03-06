#![cfg(test)]
use crate::tour::between;
use crate::{
    data::NodeKind,
    data::{DataStore, Metric},
    tour::NodeRel,
    Scalar,
};

use super::{Tour, TourOrder};

pub(crate) fn create_store(n_nodes: usize) -> DataStore<()> {
    let mut store = DataStore::<()>::with_capacity(Metric::Euc3d, n_nodes);
    for ii in 0..n_nodes {
        store.add(NodeKind::Target, vec![ii as Scalar; 3], ());
    }
    store.compute();
    store
}

pub(crate) fn test_tour_order(tour: &impl Tour, expected: &TourOrder) {
    let expected = &expected.order;
    let len = expected.len();

    assert_eq!(tour.len(), len, "Test tour len");

    for ii in 0..(expected.len() - 1) {
        let base = tour.get(expected[ii]);
        assert!(base.is_some());
        let base = base.unwrap();
        let pred = tour.predecessor(&base);
        let succ = tour.successor(&base);

        // If both orders are in the same direction, pred = targ1 and succ = targ2.
        // On the other hand, if one of them is reversed, pred = targ2 and succ = targ1.
        let targ1 = tour.get(expected[(len + ii - 1) % len]);
        let targ2 = tour.get(expected[(ii + 1) % len]);

        assert!(pred.is_some());
        assert!(
            pred == targ1 || pred == targ2,
            "Test predecessor at index = {}",
            ii
        );
        assert_eq!(NodeRel::Successor, tour.relation(&base, &pred.unwrap()));

        assert!(succ.is_some());
        assert!(
            succ == targ1 || succ == targ2,
            "Test successor at index = {}",
            ii
        );
        assert_eq!(NodeRel::Predecessor, tour.relation(&base, &succ.unwrap()));
    }
}

#[test]
fn test_between() {
    // 1 -> 2 -> 3 -> 4 -> 5
    assert!(between(1, 3, 4)); // true
    assert!(!between(1, 5, 4)); // false
    assert!(between(5, 1, 3)); // true
    assert!(!between(5, 3, 1)); // false
}

#[allow(dead_code, unused_imports)]
mod test_tll {
    use std::collections::HashMap;

    use super::*;

    use crate::tour::{
        tests::{create_store, test_tour_order},
        tll::TwoLevelList,
        STree, Tour, TourIter, TourOrder,
    };

    #[test]
    fn test_apply() {
        let mut tour = TwoLevelList::new(&create_store(10), 4);
        test_suite::apply(&mut tour);
    }

    #[test]
    fn test_total_dist() {
        let mut tour = TwoLevelList::new(&create_store(4), 3);
        test_suite::total_dist(&mut tour);
    }

    #[test]
    fn test_between() {
        let mut tour = TwoLevelList::new(&create_store(10), 3);
        test_suite::between(&mut tour);
    }

    #[test]
    fn test_flip_cases() {
        let mut tour = TwoLevelList::new(&create_store(100), 10);
        test_suite::flip(&mut tour);
    }
}

#[cfg(test)]
use float_cmp::approx_eq;

#[cfg(test)]
#[allow(dead_code)]
mod test_suite {
    use crate::{
        combine_range,
        tour::{tests::test_tour_order, Tour, TourOrder},
        tour_order, Scalar,
    };

    pub fn apply(tour: &mut impl Tour) {
        let expected = TourOrder::with_ord(vec![3, 0, 4, 1, 6, 8, 7, 9, 5, 2]);
        assert!(tour.apply(&expected).is_ok());
        test_tour_order(tour, &expected);
    }

    pub fn total_dist(tour: &mut impl Tour) {
        assert_eq!(4, tour.len());
        assert!(tour.apply(&TourOrder::with_ord(vec![0, 1, 2, 3])).is_ok());
        let r1 = 6. * (2. as Scalar).sqrt();
        crate::tour::tests::approx_eq!(f64, r1, tour.total_distance(), epsilon = 1e-10);
        crate::tour::tests::approx_eq!(f64, r1, tour.tour_order().cost(), epsilon = 1e-10);

        assert!(tour.apply(&TourOrder::with_ord(vec![1, 3, 0, 2])).is_ok());
        let r2 = 8. * (2. as Scalar).sqrt();
        crate::tour::tests::approx_eq!(f64, r2, tour.total_distance(), epsilon = 1e-10);
        crate::tour::tests::approx_eq!(f64, r2, tour.tour_order().cost(), epsilon = 1e-10);
    }

    pub fn between(tour: &mut impl Tour) {
        assert_eq!(10, tour.len());
        assert!(tour.apply(&TourOrder::with_ord((0..10).collect())).is_ok());

        //  0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9

        // All vertices reside under the same parent node.
        assert!(tour.between_at(0, 1, 2)); // true
        assert!(!tour.between_at(0, 2, 1)); // false
        assert!(!tour.between_at(2, 1, 0)); // false
        assert!(tour.between_at(2, 0, 1)); // true

        // All vertices reside under distinct parent node.
        assert!(tour.between_at(2, 3, 7)); // true
        assert!(!tour.between_at(2, 7, 3)); // true
        assert!(!tour.between_at(7, 3, 2)); // false
        assert!(tour.between_at(7, 2, 3)); // true

        // Two out of three vertices reside under the same parent node.
        assert!(tour.between_at(3, 5, 8)); // true
        assert!(!tour.between_at(3, 8, 5)); // false
        assert!(!tour.between_at(8, 5, 3)); // false
        assert!(tour.between_at(8, 3, 5)); // true

        // Reverse [3 4 5]
        assert!(tour.between_at(3, 4, 5)); // true
        assert!(!tour.between_at(5, 4, 3)); // false

        tour.flip_at(2, 3, 5, 6);

        assert!(!tour.between_at(3, 4, 5)); // false
        assert!(tour.between_at(5, 4, 3)); // true

        assert!(!tour.between_at(3, 5, 8)); // false
        assert!(tour.between_at(3, 8, 5)); // true
        assert!(tour.between_at(8, 5, 3)); // true
        assert!(!tour.between_at(8, 3, 5)); // false
    }

    pub fn flip(tour: &mut impl Tour) {
        flip_1(tour);
        flip_2(tour);
        flip_3(tour);
        flip_4(tour);
    }

    // Test flip case: New paths lie within the same segment.
    fn flip_1(tour: &mut impl Tour) {
        let n_nodes = 100;
        assert_eq!(n_nodes, tour.len());
        assert!(tour.apply(&tour_order!(0..n_nodes)).is_ok());

        tour.rev();
        tour.flip_at(9, 8, 4, 3);
        test_tour_order(tour, &tour_order!(0..4, (4..9).rev(), 9..n_nodes));

        tour.rev();

        tour.flip_at(3, 8, 4, 9);
        test_tour_order(tour, &tour_order!(0..n_nodes));

        tour.flip_at(8, 9, 3, 4);
        test_tour_order(tour, &tour_order!(0..4, (4..9).rev(), 9..n_nodes));

        tour.rev();
        tour.flip_at(9, 4, 8, 3);
        test_tour_order(tour, &TourOrder::with_nat_ord(n_nodes));

        // Reverses the entire segment.
        tour.flip_at(10, 9, 20, 19);
        test_tour_order(tour, &tour_order!(0..10, (10..20).rev(), 20..n_nodes));

        tour.rev();
        tour.flip_at(10, 20, 9, 19);
        test_tour_order(tour, &TourOrder::with_nat_ord(n_nodes));
    }

    // Test flip case: New paths consist of a sequence of consecutive segments.
    // This test focuses on inner reverse.
    fn flip_2(tour: &mut impl Tour) {
        let n_nodes = 100;
        assert_eq!(n_nodes, tour.len());
        assert!(tour.apply(&TourOrder::with_nat_ord(n_nodes)).is_ok());

        tour.flip_at(9, 10, 39, 40);
        test_tour_order(tour, &tour_order!(0..10, (10..40).rev(), 40..n_nodes));

        tour.rev();
        tour.flip_at(40, 10, 39, 9);
        test_tour_order(tour, &TourOrder::with_nat_ord(n_nodes));

        tour.flip_at(30, 29, 10, 9);
        test_tour_order(tour, &tour_order!(0..10, (10..30).rev(), 30..n_nodes));

        tour.flip_at(29, 9, 30, 10);
        test_tour_order(tour, &TourOrder::with_nat_ord(n_nodes));
    }

    // Test flip case: New paths consist of a sequence of consecutive segments.
    // This test focuses on outer reverse.
    fn flip_3(tour: &mut impl Tour) {
        let n_nodes = 100;
        assert_eq!(n_nodes, tour.len());
        assert!(tour.apply(&TourOrder::with_nat_ord(n_nodes)).is_ok());

        tour.flip_at(9, 10, 89, 90);
        test_tour_order(
            tour,
            &tour_order!((90..n_nodes).rev(), 10..90, (0..10).rev()),
        );

        tour.flip_at(90, 10, 89, 9);
        test_tour_order(tour, &TourOrder::with_nat_ord(n_nodes));

        tour.flip_at(89, 90, 9, 10);
        tour.rev();
        test_tour_order(
            tour,
            &tour_order!((90..n_nodes).rev(), 10..90, (0..10).rev()),
        );

        tour.flip_at(9, 89, 10, 90);
        test_tour_order(tour, &TourOrder::with_nat_ord(n_nodes));

        tour.flip_at(80, 79, 90, 89);
        tour.rev();
        tour.flip_at(9, 10, 79, 89);

        test_tour_order(
            tour,
            &tour_order!(80..90, 10..80, (0..10).rev(), (90..n_nodes).rev()),
        );
    }

    // Test flip case: Vertices are positioned in the middle of segments.
    fn flip_4(tour: &mut impl Tour) {
        let n_nodes = 100;
        assert_eq!(n_nodes, tour.len());

        flip_4_d1_forward_move_back(tour);
        flip_4_d1_reverse_move_front(tour);
        flip_4_d2_forward_move_front(tour);
        flip_4_d2_reverse_move_back(tour);
    }

    // Called by flip_4().
    // d1 <= d2 in both from- and to-sides and the segments are forward.
    // Affected nodes will be moved to corresponding prev-segments.
    // prev of from-side: forward
    // prev of to-side: reverse
    fn flip_4_d1_forward_move_back(tour: &mut impl Tour) {
        assert!(tour.apply(&TourOrder::with_nat_ord(tour.len())).is_ok());
        test_tour_order(tour, &TourOrder::with_nat_ord(tour.len()));

        // Reverse prev of to-side.
        tour.flip_at(59, 60, 69, 70);

        tour.flip_at(33, 34, 72, 73);
        test_tour_order(
            tour,
            &tour_order!(
                0..34,
                (70..73).rev(),
                60..70,
                (34..60).rev(),
                73..tour.len()
            ),
        );
    }

    // Called by flip_4().
    // Corresponds to d1 <= d2 in both from- and to-sides and the segments are reversed.
    // Affected nodes will be moved to corresponding next-segments.
    // next of from-side: forward
    // next of to-side: reverse
    fn flip_4_d1_reverse_move_front(tour: &mut impl Tour) {
        assert!(tour.apply(&TourOrder::with_nat_ord(tour.len())).is_ok());
        test_tour_order(tour, &TourOrder::with_nat_ord(tour.len()));

        // Reverse from- and to-side.
        tour.flip_at(29, 30, 39, 40);
        tour.flip_at(59, 60, 69, 70);

        // Reverse next of to-side.
        tour.flip_at(60, 70, 79, 80);

        // Flip operation.
        tour.flip_at(34, 33, 63, 62);
        test_tour_order(
            tour,
            &tour_order!(
                0..30,
                (34..40).rev(),
                63..70,
                (40..60).rev(),
                30..34,
                (60..63).rev(),
                (70..80).rev(),
                80..tour.len()
            ),
        );
    }

    // Called by flip_4().
    // d1 > d2 in both from- and to-sides and the segments are forward.
    // Affected nodes will be moved to corresponding next-segments.
    // next of from-side: forward
    // next of to-side: reverse
    fn flip_4_d2_forward_move_front(tour: &mut impl Tour) {
        assert!(tour.apply(&TourOrder::with_nat_ord(tour.len())).is_ok());
        test_tour_order(tour, &TourOrder::with_nat_ord(tour.len()));

        // Reverse next of to-side.
        tour.flip_at(69, 70, 79, 80);

        tour.flip_at(36, 37, 67, 68);
        test_tour_order(
            tour,
            &tour_order!(
                0..37,
                (37..68).rev(),
                68..70,
                (70..80).rev(),
                80..tour.len()
            ),
        );
    }

    // Called by flip_4().
    // Corresponds to d1 > d2 in both from- and to-sides and the segments are reversed.
    // Affected nodes will be moved to corresponding prev-segments.
    // prev of from-side: forward
    // prev of to-side: reverse
    fn flip_4_d2_reverse_move_back(tour: &mut impl Tour) {
        assert!(tour.apply(&TourOrder::with_nat_ord(tour.len())).is_ok());
        test_tour_order(tour, &TourOrder::with_nat_ord(tour.len()));

        // Reverse from- and to-side.
        tour.flip_at(29, 30, 39, 40);
        tour.flip_at(69, 70, 79, 80);

        // Reverse prev of to-side.
        tour.flip_at(59, 60, 69, 79);

        // Flip operation.
        tour.flip_at(37, 36, 68, 67);

        test_tour_order(
            tour,
            &tour_order!(
                0..30,
                (37..40).rev(),
                68..70,
                (40..60).rev(),
                30..37,
                (60..68).rev(),
                (70..80).rev(),
                80..tour.len()
            ),
        );
    }
}
