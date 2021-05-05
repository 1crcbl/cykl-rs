use crate::{metric::MetricKind, node::Container, Scalar};
#[allow(unused_imports)]
use crate::tour::between;

use super::{Tour, TourOrder};

#[allow(dead_code)]
pub fn create_container(n_nodes: usize) -> Container {
    let mut container = Container::new(MetricKind::Euc2d);
    for ii in 0..n_nodes {
        container.add(ii as Scalar, ii as Scalar, ii as Scalar);
    }
    container
}

#[allow(dead_code)]
pub fn test_tree_order(tour: &impl Tour, expected: &TourOrder) {
    let expected = &expected.order;
    let len = expected.len();
    
    assert_eq!(tour.size(), len);
    assert_eq!(tour.get(expected[0]), tour.next_at(expected[len - 1]));
    assert_eq!(tour.get(expected[len - 1]), tour.prev_at(expected[0]));

    for ii in 1..(expected.len() - 1) {
        assert_eq!(tour.get(expected[ii]), tour.prev_at(expected[ii + 1]));
        assert_eq!(tour.get(expected[ii + 1]), tour.next_at(expected[ii]));
    }

    assert_eq!(
        tour.get(expected[0]),
        tour.next(tour.get(expected[len - 1]).unwrap())
    );
    assert_eq!(
        tour.get(expected[len - 1]),
        tour.prev(tour.get(expected[0]).unwrap())
    );
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
        let container = create_container(10);
        let mut tour = Array::new(&container);
        let expected = TourOrder::new(vec![3, 0, 4, 1, 6, 8, 7, 9, 5, 2]);
        tour.apply(&expected);
        test_tree_order(&tour, &expected);
    }

    #[test]
    fn test_total_dist() {
        let container = create_container(4);
        let mut tour = Array::new(&container);
        tour.apply(&TourOrder::new(vec![0, 1, 2, 3]));
        assert_eq!(6. * (2. as Scalar).sqrt(), tour.total_distance());

        tour.apply(&TourOrder::new(vec![1, 3, 0, 2]));
        assert_eq!(8. * (2. as Scalar).sqrt(), tour.total_distance());
    }

    #[test]
    fn test_next() {
        let container = create_container(10);
        let tour = Array::new(&container);

        // [2] -> [3]
        assert_eq!(tour.get(3).unwrap(), tour.next_at(2).unwrap());

        // [4] -> [0]
        assert_eq!(tour.get(0).unwrap(), tour.next_at(9).unwrap());
    }

    #[test]
    fn test_prev() {
        let container = create_container(10);
        let tour = Array::new(&container);

        // [2] -> [3]
        assert_eq!(tour.get(2).unwrap(), tour.prev_at(3).unwrap());

        // [4] -> [0]
        assert_eq!(tour.get(9).unwrap(), tour.prev_at(0).unwrap());
    }

    #[test]
    fn test_swap() {
        let container = create_container(10);
        let mut tour = Array::new(&container);

        // [0] <-> [9]
        tour.swap_at(0, 9);
        test_tree_order(&tour, &TourOrder::new(vec![9, 1, 2, 3, 4, 5, 6, 7, 8, 0]));
    }

    #[test]
    fn test_flip_case_1() {
        let container = create_container(10);
        let mut tour = Array::new(&container);

        tour.flip_at(2, 3, 6, 7);
        let expected = vec![0, 1, 2, 6, 5, 4, 3, 7, 8, 9];
        assert_eq!(&expected, tour.tracker());
    }

    #[test]
    fn test_flip_case_2() {
        let container = create_container(10);
        let mut tour = Array::new(&container);

        // Expected: 0 - 1 - 9 - 8 - 7 - 6 - 5 - 4 - 3 - 2
        tour.flip_at(9, 0, 1, 2);
        let expected = vec![0, 1, 9, 8, 7, 6, 5, 4, 3, 2];
        assert_eq!(&expected, tour.tracker());
    }

    #[test]
    fn test_between() {
        let container = create_container(10);
        let tour = Array::new(&container);

        // from < to
        assert!(tour.between_at(2, 5, 8));
        assert!(!tour.between_at(2, 1, 8));
        assert!(tour.between_at(2, 2, 8));
        assert!(tour.between_at(2, 8, 8));

        // from > to
        assert!(tour.between_at(8, 1, 2));
        assert!(!tour.between_at(8, 5, 2));
        assert!(tour.between_at(8, 2, 2));
        assert!(tour.between_at(8, 8, 2));

        // from == to
        assert!(tour.between_at(2, 2, 2));
        assert!(!tour.between_at(2, 8, 2));
    }
}

#[allow(dead_code, unused_imports)]
mod tests_tlt {
    use crate::tour::TwoLevelTree;

    use super::*;

    #[test]
    fn test_apply() {
        let container = create_container(10);
        let mut tour = TwoLevelTree::new(&container, 4);
        let expected = TourOrder::new(vec![3, 0, 4, 1, 6, 8, 7, 9, 5, 2]);
        tour.apply(&expected);
        test_tree_order(&tour, &expected);
    }

    #[test]
    fn test_between() {
        let n_nodes = 10;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);
        tree.apply(&TourOrder::new((0..n_nodes).collect()));

        //  0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9

        // All vertices reside under the same parent node.
        assert!(tree.between_at(0, 1, 2)); // true
        assert!(!tree.between_at(0, 2, 1)); // false
        assert!(!tree.between_at(2, 1, 0)); // false
        assert!(tree.between_at(2, 0, 1)); // true

        // All vertices reside under distinct parent node.
        assert!(tree.between_at(2, 3, 7)); // true
        assert!(!tree.between_at(2, 7, 3)); // true
        assert!(!tree.between_at(7, 3, 2)); // false
        assert!(tree.between_at(7, 2, 3)); // true

        // Two out of three vertices reside under the same parent node.
        assert!(tree.between_at(3, 5, 8)); // true
        assert!(!tree.between_at(3, 8, 5)); // false
        assert!(!tree.between_at(8, 5, 3)); // false
        assert!(tree.between_at(8, 3, 5)); // true

        // Reverse [3 4 5]
        assert!(tree.between_at(3, 4, 5)); // true
        assert!(!tree.between_at(5, 4, 3)); // false

        tree.parent(1).borrow_mut().reverse();

        assert!(!tree.between_at(3, 4, 5)); // false
        assert!(tree.between_at(5, 4, 3)); // true

        assert!(!tree.between_at(3, 5, 8)); // false
        assert!(tree.between_at(3, 8, 5)); // true
        assert!(tree.between_at(8, 5, 3)); // true
        assert!(!tree.between_at(8, 3, 5)); // false
    }

    #[test]
    fn test_total_dist() {
        let n_nodes = 4;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);

        tree.apply(&TourOrder::new(vec![0, 1, 2, 3]));
        assert_eq!(6. * (2. as Scalar).sqrt(), tree.total_distance());

        tree.apply(&TourOrder::new(vec![1, 3, 0, 2]));
        assert_eq!(8. * (2. as Scalar).sqrt(), tree.total_distance());
    }

    // Test flip case: New paths lie within the same segment.
    #[test]
    fn test_flip_1() {
        let n_nodes = 50;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 10);
        tree.apply(&TourOrder::new((0..n_nodes).collect()));

        tree.flip_at(3, 4, 8, 9);
        let mut expected = vec![0, 1, 2, 3, 8, 7, 6, 5, 4, 9];
        expected.append(&mut (10..n_nodes).collect());
        test_tree_order(&tree, &TourOrder::new(expected));

        tree.flip_at(3, 8, 4, 9);
        test_tree_order(&tree, &TourOrder::new((0..n_nodes).collect()));

        tree.flip_at(8, 9, 3, 4);
        let mut expected = vec![0, 1, 2, 3, 8, 7, 6, 5, 4, 9];
        expected.append(&mut (10..n_nodes).collect());
        test_tree_order(&tree, &TourOrder::new(expected));

        tree.flip_at(4, 9, 3, 8);
        let mut expected = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        expected.append(&mut (10..n_nodes).collect());
        test_tree_order(&tree, &TourOrder::new(expected));

        // Reverses the entire segment.
        tree.flip_at(9, 10, 19, 20);
        let mut expected: Vec<usize> = (0..10).collect();
        expected.append(&mut (10..20).rev().collect());
        expected.append(&mut (20..n_nodes).collect());
        test_tree_order(&tree, &TourOrder::new(expected));

        tree.parent(1).borrow_mut().reverse();
        test_tree_order(&tree, &TourOrder::new((0..n_nodes).collect()));
    }

    // Test flip case: New paths consist of a sequence of consecutive segments.
    // This test focuses on inner reverse.
    #[test]
    fn test_flip_2() {
        let n_nodes = 50;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 10);
        tree.apply(&TourOrder::new((0..n_nodes).collect()));

        tree.flip_at(9, 10, 29, 30);
        let mut expected: Vec<usize> = (0..10).collect();
        expected.append(&mut (20..30).rev().collect());
        expected.append(&mut (10..20).rev().collect());
        expected.append(&mut (30..n_nodes).collect());
        test_tree_order(&tree, &TourOrder::new(expected));

        tree.flip_at(10, 30, 9, 29);
        test_tree_order(&tree, &TourOrder::new((0..n_nodes).collect()));

        tree.flip_at(29, 30, 9, 10);
        let mut expected: Vec<usize> = (0..10).collect();
        expected.append(&mut (20..30).rev().collect());
        expected.append(&mut (10..20).rev().collect());
        expected.append(&mut (30..n_nodes).collect());
        test_tree_order(&tree, &TourOrder::new(expected));

        tree.flip_at(9, 29, 10, 30);
        test_tree_order(&tree, &TourOrder::new((0..n_nodes).collect()));

        tree.parent(1).borrow_mut().reverse();

        tree.flip_at(9, 19, 29, 30);
        let mut expected: Vec<usize> = (0..10).collect();
        expected.append(&mut (20..30).rev().collect());
        expected.append(&mut (10..20).collect());
        expected.append(&mut (30..n_nodes).collect());
        test_tree_order(&tree, &TourOrder::new(expected));
    }

    // Test flip case: New paths consist of a sequence of consecutive segments.
    // This test focuses on outer reverse.
    #[test]
    fn test_flip_3() {
        let n_nodes = 100;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 10);
        tree.apply(&TourOrder::new((0..n_nodes).collect()));

        let mut expected: Vec<usize> = (90..n_nodes).rev().collect();
        expected.append(&mut (10..90).collect());
        expected.append(&mut (0..10).rev().collect());
        tree.flip_at(9, 10, 89, 90);
        test_tree_order(&tree, &TourOrder::new(expected));

        tree.flip_at(90, 10, 89, 9);
        test_tree_order(&tree, &TourOrder::new((0..n_nodes).collect()));

        let mut expected: Vec<usize> = (90..n_nodes).rev().collect();
        expected.append(&mut (10..90).collect());
        expected.append(&mut (0..10).rev().collect());
        tree.flip_at(89, 90, 9, 10);
        test_tree_order(&tree, &TourOrder::new(expected));

        tree.flip_at(89, 9, 90, 10);
        test_tree_order(&tree, &TourOrder::new((0..n_nodes).collect()));

        tree.parent(8).borrow_mut().reverse();

        let mut expected: Vec<usize> = (80..90).collect();
        expected.append(&mut (10..80).collect());
        expected.append(&mut (0..10).rev().collect());
        expected.append(&mut (90..n_nodes).rev().collect());
        tree.flip_at(9, 10, 79, 89);
        test_tree_order(&tree, &TourOrder::new(expected));
    }

    #[test]
    fn test_parent_reverse() {
        let n_nodes = 10;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);

        tree.apply(&TourOrder::new((0..n_nodes).collect()));

        // 0 -> 1 -> 2 -> 5 -> 4 -> 3 -> 6 -> 7 -> 8 -> 9
        tree.parent(1).borrow_mut().reverse();
        test_tree_order(&tree, &TourOrder::new(vec![0, 1, 2, 5, 4, 3, 6, 7, 8, 9]));

        // 0 -> 1 -> 2 -> 5 -> 4 -> 3 -> 8 -> 7 -> 6 -> 9
        tree.parent(2).borrow_mut().reverse();
        let order = TourOrder::new(vec![0, 1, 2, 5, 4, 3, 8, 7, 6, 9]);
        test_tree_order(&tree, &order);

        tree.parent(3).borrow_mut().reverse();
        test_tree_order(&tree, &order);

        // 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9
        tree.parent(1).borrow_mut().reverse();
        tree.parent(2).borrow_mut().reverse();
        test_tree_order(&tree, &TourOrder::new((0..10).collect()));
    }
}