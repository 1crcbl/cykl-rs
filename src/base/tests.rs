#[allow(dead_code, unused_imports)]
mod metric_tests {
    use crate::{Scalar, base::repo::DataNode};
    use crate::base::repo::{dist_euc_2d, dist_euc_3d};

    #[test]
    fn test_euc_2d() {
        let (a, b) = create_node_pair();
        assert_eq!((34 as Scalar).sqrt(), dist_euc_2d(&a, &b));
    }

    #[test]
    fn test_euc_3d() {
        let (a, b) = create_node_pair();
        assert_eq!((35 as Scalar).sqrt(), dist_euc_3d(&a, &b));
    }

    fn create_node_pair() -> (DataNode, DataNode) {
        let a = DataNode::new(0, 1., 2., 3.);
        let b = DataNode::new(1, 6., 5., 4.);
        (a, b)
    }
}