#[allow(unused_imports)]
use crate::tour::between;
use crate::{MetricKind, Repo, RepoBuilder, Scalar};

use super::{Tour, TourOrder};

#[allow(dead_code)]
pub fn create_repo(n_nodes: usize) -> Repo {
    let builder = RepoBuilder::new(MetricKind::Euc2d).capacity(n_nodes);
    let mut repo = builder.build();
    for ii in 0..n_nodes {
        repo.add(ii as Scalar, ii as Scalar, ii as Scalar);
    }
    repo
}

#[allow(dead_code)]
pub fn test_tour_order(tour: &impl Tour, expected: &TourOrder) {
    let expected = &expected.order;
    let len = expected.len();

    assert_eq!(tour.len(), len, "Test tour len");

    for ii in 0..(expected.len() - 1) {
        assert_eq!(
            tour.get(expected[(len + ii - 1) % len]),
            tour.predecessor_at(expected[ii]),
            "Test predecessor at index = {}",
            ii
        );
        assert_eq!(
            tour.get(expected[(ii + 1) % len]),
            tour.successor_at(expected[ii]),
            "Test successor at index = {}",
            ii
        );
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
mod tests_array {
    use crate::tour::Array;

    use super::*;

    #[test]
    fn test_apply() {
        let repo = create_repo(10);
        let mut tour = Array::new(&repo);
        test_suite::apply(&mut tour);
    }

    #[test]
    fn test_total_dist() {
        let repo = create_repo(4);
        let mut tour = Array::new(&repo);
        test_suite::total_dist(&mut tour);
    }

    #[test]
    #[ignore = "requires reimpl"]
    fn test_between() {
        let repo = create_repo(10);
        let mut tour = Array::new(&repo);
        test_suite::between(&mut tour);
    }

    #[test]
    #[ignore = "requires reimpl"]
    fn test_flip_cases() {
        let repo = create_repo(100);
        let mut tour = Array::new(&repo);
        test_suite::flip(&mut tour);
    }
}

#[allow(dead_code, unused_imports)]
mod test_tll {
    use std::collections::HashMap;

    use super::*;

    use crate::{
        tour::{
            tests::{create_repo, test_tour_order},
            tll::TwoLevelList,
            STree, Tour, TourIter, TourOrder,
        },
        MatrixKind,
    };

    #[test]
    fn test_apply() {
        let repo = create_repo(10);
        let mut tour = TwoLevelList::new(&repo, 4);
        test_suite::apply(&mut tour);
    }

    #[test]
    fn test_total_dist() {
        let repo = create_repo(4);
        let mut tour = TwoLevelList::new(&repo, 3);
        test_suite::total_dist(&mut tour);
    }

    #[test]
    fn test_between() {
        let repo = create_repo(10);
        let mut tour = TwoLevelList::new(&repo, 3);
        test_suite::between(&mut tour);
    }

    #[test]
    fn test_flip_cases() {
        let repo = create_repo(100);
        let mut tour = TwoLevelList::new(&repo, 10);
        test_suite::flip(&mut tour);
    }

    #[test]
    fn test_build_mst() {
        // Data is taken from Wikipedia article for MST.
        // https://en.wikipedia.org/wiki/Minimum_spanning_tree

        let costs = vec![
            vec![0., 1., 0., 4., 3., 0.],
            vec![0., 0., 4., 2., 0.],
            vec![0., 0., 4., 5.],
            vec![0., 4., 0.],
            vec![0., 7.],
            vec![0.],
        ];

        let repo = RepoBuilder::new(MetricKind::Euc2d)
            .costs(costs, MatrixKind::Upper)
            .build();

        let mut tour = TwoLevelList::new(&repo, 6);
        tour.build_mst();

        let mut result = HashMap::new();

        for (idx, node) in tour.itr().enumerate() {
            unsafe {
                if idx == 0 {
                    result.insert(idx, None);
                } else {
                    let parent = &node.mst_parent;
                    assert!(parent.is_some());
                    result.insert(idx, Some((*parent.unwrap().as_ptr()).index()));
                }
            }
        }

        // There are many possiblities of MST for a given graph.
        // Here we use the second MST output shown in the Wikipedia article.
        let expected: HashMap<usize, Option<usize>> = [
            (0, None),
            (1, Some(0)),
            (4, Some(1)),
            (3, Some(0)),
            (2, Some(4)),
            (5, Some(2)),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(expected, result);
    }

    #[test]
    // TODO: test vec elements.
    fn test_gen_cands() {
        let repo = create_repo(20);
        let mut tour = TwoLevelList::with_default_order(&repo, 10);

        let k = 6;
        tour.gen_cands(k);

        for base in tour.itr() {
            let mut results = Vec::with_capacity(k);

            unsafe {
                for targ in &base.cands {
                    assert!(targ.is_some());
                    results.push((*targ.unwrap().as_ptr()).index());
                }
            }

            assert_eq!(k, results.len());
        }
    }
}

#[allow(dead_code)]
mod test_suite {
    use crate::{
        tour::{tests::test_tour_order, Tour, TourOrder},
        Scalar,
    };

    pub fn apply(tour: &mut impl Tour) {
        let expected = TourOrder::with_ord(vec![3, 0, 4, 1, 6, 8, 7, 9, 5, 2]);
        tour.apply(&expected);
        test_tour_order(tour, &expected);
    }

    pub fn total_dist(tour: &mut impl Tour) {
        assert_eq!(4, tour.len());
        tour.apply(&TourOrder::with_ord(vec![0, 1, 2, 3]));
        assert_eq!(6. * (2. as Scalar).sqrt(), tour.total_distance());

        tour.apply(&TourOrder::with_ord(vec![1, 3, 0, 2]));
        assert_eq!(8. * (2. as Scalar).sqrt(), tour.total_distance());
    }

    pub fn between(tour: &mut impl Tour) {
        assert_eq!(10, tour.len());
        tour.apply(&TourOrder::with_ord((0..10).collect()));

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
        tour.apply(&TourOrder::with_ord((0..n_nodes).collect()));

        tour.flip_at(3, 4, 8, 9);
        let mut expected = vec![0, 1, 2, 3, 8, 7, 6, 5, 4, 9];
        expected.append(&mut (10..n_nodes).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));

        tour.flip_at(3, 8, 4, 9);
        test_tour_order(tour, &TourOrder::with_ord((0..n_nodes).collect()));

        tour.flip_at(8, 9, 3, 4);
        let mut expected = vec![0, 1, 2, 3, 8, 7, 6, 5, 4, 9];
        expected.append(&mut (10..n_nodes).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));

        tour.flip_at(4, 9, 3, 8);
        let mut expected = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        expected.append(&mut (10..n_nodes).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));

        // Reverses the entire segment.
        tour.flip_at(9, 10, 19, 20);
        let mut expected: Vec<usize> = (0..10).collect();
        expected.append(&mut (10..20).rev().collect());
        expected.append(&mut (20..n_nodes).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));

        tour.flip_at(10, 20, 9, 19);
        test_tour_order(tour, &TourOrder::with_ord((0..n_nodes).collect()));
    }

    // Test flip case: New paths consist of a sequence of consecutive segments.
    // This test focuses on inner reverse.
    fn flip_2(tour: &mut impl Tour) {
        let n_nodes = 100;
        assert_eq!(n_nodes, tour.len());
        tour.apply(&TourOrder::with_ord((0..n_nodes).collect()));

        tour.flip_at(9, 10, 39, 40);
        let mut expected: Vec<usize> = (0..10).collect();
        expected.append(&mut (10..40).rev().collect());
        expected.append(&mut (40..n_nodes).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));

        tour.flip_at(10, 40, 9, 39);
        test_tour_order(tour, &TourOrder::with_ord((0..n_nodes).collect()));

        tour.flip_at(29, 30, 9, 10);
        let mut expected: Vec<usize> = (0..10).collect();
        expected.append(&mut (10..30).rev().collect());
        expected.append(&mut (30..n_nodes).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));

        tour.flip_at(9, 29, 10, 30);
        test_tour_order(tour, &TourOrder::with_ord((0..n_nodes).collect()));
    }

    // Test flip case: New paths consist of a sequence of consecutive segments.
    // This test focuses on outer reverse.
    fn flip_3(tour: &mut impl Tour) {
        let n_nodes = 100;
        assert_eq!(n_nodes, tour.len());
        tour.apply(&TourOrder::with_ord((0..n_nodes).collect()));

        let mut expected: Vec<usize> = (90..n_nodes).rev().collect();
        expected.append(&mut (10..90).collect());
        expected.append(&mut (0..10).rev().collect());
        tour.flip_at(9, 10, 89, 90);
        test_tour_order(tour, &TourOrder::with_ord(expected));

        tour.flip_at(90, 10, 89, 9);
        test_tour_order(tour, &TourOrder::with_ord((0..n_nodes).collect()));

        let mut expected: Vec<usize> = (90..n_nodes).rev().collect();
        expected.append(&mut (10..90).collect());
        expected.append(&mut (0..10).rev().collect());
        tour.flip_at(89, 90, 9, 10);
        test_tour_order(tour, &TourOrder::with_ord(expected));

        tour.flip_at(89, 9, 90, 10);
        test_tour_order(tour, &TourOrder::with_ord((0..n_nodes).collect()));

        tour.flip_at(79, 80, 89, 90);
        tour.flip_at(9, 10, 79, 89);

        let mut expected: Vec<usize> = (80..90).collect();
        expected.append(&mut (10..80).collect());
        expected.append(&mut (0..10).rev().collect());
        expected.append(&mut (90..n_nodes).rev().collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));
    }

    // Test flip case: Vertices are positioned in the middle of segments.
    fn flip_4(tour: &mut impl Tour) {
        let n_nodes = 100;
        assert_eq!(n_nodes, tour.len());
        let order0 = TourOrder::with_ord((0..n_nodes).collect());
        tour.apply(&order0);
        test_tour_order(tour, &order0);

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
        tour.apply(&TourOrder::with_ord((0..tour.len()).collect()));
        test_tour_order(tour, &TourOrder::with_ord((0..tour.len()).collect()));

        // Reverse prev of to-side.
        tour.flip_at(59, 60, 69, 70);

        tour.flip_at(33, 34, 72, 73);

        let mut expected: Vec<usize> = (0..34).collect();
        expected.append(&mut (70..73).rev().collect());
        expected.append(&mut (60..70).collect());
        expected.append(&mut (34..60).rev().collect());
        expected.append(&mut (73..tour.len()).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));
    }

    // Called by flip_4().
    // Corresponds to d1 <= d2 in both from- and to-sides and the segments are reversed.
    // Affected nodes will be moved to corresponding next-segments.
    // next of from-side: forward
    // next of to-side: reverse
    fn flip_4_d1_reverse_move_front(tour: &mut impl Tour) {
        tour.apply(&TourOrder::with_ord((0..tour.len()).collect()));
        test_tour_order(tour, &TourOrder::with_ord((0..tour.len()).collect()));

        // Reverse from- and to-side.
        tour.flip_at(29, 30, 39, 40);
        tour.flip_at(59, 60, 69, 70);

        // Reverse next of to-side.
        tour.flip_at(60, 70, 79, 80);

        // Flip operation.
        tour.flip_at(34, 33, 63, 62);

        let mut expected: Vec<usize> = (0..30).collect();
        expected.append(&mut (34..40).rev().collect());
        expected.append(&mut (63..70).collect());
        expected.append(&mut (40..60).rev().collect());
        expected.append(&mut (30..34).collect());
        expected.append(&mut (60..63).rev().collect());
        expected.append(&mut (70..80).rev().collect());
        expected.append(&mut (80..tour.len()).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));
    }

    // Called by flip_4().
    // d1 > d2 in both from- and to-sides and the segments are forward.
    // Affected nodes will be moved to corresponding next-segments.
    // next of from-side: forward
    // next of to-side: reverse
    fn flip_4_d2_forward_move_front(tour: &mut impl Tour) {
        tour.apply(&TourOrder::with_ord((0..tour.len()).collect()));
        test_tour_order(tour, &TourOrder::with_ord((0..tour.len()).collect()));

        // Reverse next of to-side.
        tour.flip_at(69, 70, 79, 80);

        tour.flip_at(36, 37, 67, 68);

        let mut expected: Vec<usize> = (0..37).collect();
        expected.append(&mut (37..68).rev().collect());
        expected.append(&mut (68..70).collect());
        expected.append(&mut (70..80).rev().collect());
        expected.append(&mut (80..tour.len()).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));
    }

    // Called by flip_4().
    // Corresponds to d1 > d2 in both from- and to-sides and the segments are reversed.
    // Affected nodes will be moved to corresponding prev-segments.
    // prev of from-side: forward
    // prev of to-side: reverse
    fn flip_4_d2_reverse_move_back(tour: &mut impl Tour) {
        tour.apply(&TourOrder::with_ord((0..tour.len()).collect()));
        test_tour_order(tour, &TourOrder::with_ord((0..tour.len()).collect()));

        // Reverse from- and to-side.
        tour.flip_at(29, 30, 39, 40);
        tour.flip_at(69, 70, 79, 80);

        // Reverse prev of to-side.
        tour.flip_at(59, 60, 69, 79);

        // Flip operation.
        tour.flip_at(37, 36, 68, 67);

        let mut expected: Vec<usize> = (0..30).collect();
        expected.append(&mut (37..40).rev().collect());
        expected.append(&mut (68..70).collect());
        expected.append(&mut (40..60).rev().collect());
        expected.append(&mut (30..37).collect());
        expected.append(&mut (60..68).rev().collect());
        expected.append(&mut (70..80).rev().collect());
        expected.append(&mut (80..tour.len()).collect());
        test_tour_order(tour, &TourOrder::with_ord(expected));
    }
}
