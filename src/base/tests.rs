#[allow(dead_code, unused_imports)]
mod metric_tests {
    use crate::base::repo::{euc_2d, euc_3d};
    use crate::{base::repo::DataNode, Scalar};

    #[test]
    fn test_euc_2d() {
        let (a, b) = create_node_pair();
        assert_eq!((34 as Scalar).sqrt(), euc_2d(&a, &b));
    }

    #[test]
    fn test_euc_3d() {
        let (a, b) = create_node_pair();
        assert_eq!((35 as Scalar).sqrt(), euc_3d(&a, &b));
    }

    fn create_node_pair() -> (DataNode, DataNode) {
        let a = DataNode::new(0, 1., 2., 3.);
        let b = DataNode::new(1, 6., 5., 4.);
        (a, b)
    }
}

#[allow(dead_code, unused_imports, unused_macros)]
mod repo_tests {
    use crate::RepoBuilder;
    use std::cmp::{max, min};

    macro_rules! full_costs {
        () => {
            vec![
                vec![0., 1., 2., 3., 4., 5.],
                vec![1., 0., 6., 7., 8., 9.],
                vec![2., 6., 0., 10., 11., 12.],
                vec![3., 7., 10., 0., 13., 14.],
                vec![4., 8., 11., 13., 0., 15.],
                vec![5., 9., 12., 14., 15., 0.],
            ];
        };
    }

    #[test]
    fn test_builder_costs_full() {
        let costs = full_costs!();

        let repo = RepoBuilder::new(crate::MetricKind::Euc2d)
            .costs(costs.clone(), crate::MatrixKind::Full)
            .build();
        for (ridx, row) in costs.iter().enumerate() {
            for (cidx, val) in row.iter().enumerate() {
                assert_eq!(
                    *val,
                    repo.distance_at(ridx, cidx),
                    "Test distance {} - {}",
                    ridx,
                    cidx
                );
            }
        }
    }

    #[test]
    fn test_builder_costs_upper() {
        let costs = vec![
            vec![0., 1., 2., 3., 4., 5.],
            vec![0., 6., 7., 8., 9.],
            vec![0., 10., 11., 12.],
            vec![0., 13., 14.],
            vec![0., 15.],
            vec![0.],
        ];

        let costs_full = full_costs!();

        let repo = RepoBuilder::new(crate::MetricKind::Euc2d)
            .costs(costs, crate::MatrixKind::Upper)
            .build();

        for ridx in 0..costs_full.len() {
            for cidx in 0..costs_full.len() {
                let (r, c) = (min(ridx, cidx), max(ridx, cidx));
                assert_eq!(
                    costs_full[r][c],
                    repo.distance_at(ridx, cidx),
                    "Test distance {} - {}",
                    ridx,
                    cidx
                );
            }
        }
    }

    #[test]
    fn test_builder_costs_lower() {
        let costs = vec![
            vec![0.],
            vec![1., 0.],
            vec![2., 6., 0.],
            vec![3., 7., 10., 0.],
            vec![4., 8., 11., 13., 0.],
            vec![5., 9., 12., 14., 15., 0.],
        ];

        let costs_full = full_costs!();

        let repo = RepoBuilder::new(crate::MetricKind::Euc2d)
            .costs(costs, crate::MatrixKind::Lower)
            .build();

        for ridx in 0..costs_full.len() {
            for cidx in 0..costs_full.len() {
                let (r, c) = (min(ridx, cidx), max(ridx, cidx));
                assert_eq!(
                    costs_full[r][c],
                    repo.distance_at(ridx, cidx),
                    "Test distance {} - {}",
                    ridx,
                    cidx
                );
            }
        }
    }
}
