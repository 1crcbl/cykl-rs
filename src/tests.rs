#![cfg(test)]
use float_cmp::approx_eq;

use crate::data::{DataStore, Metric, NodeKind};

#[test]
fn test_metric() {
    let len = 10;
    let mut ds = DataStore::with_capacity(Metric::Euc2d, len);
    (0..len).for_each(|ii| {
        ds.add(NodeKind::Target, vec![ii as f64; 2], ());
    });

    ds.compute();

    for ii in 0..len {
        for jj in 0..len {
            let exp = Metric::Euc2d.cost(&vec![ii as f64; 2], &vec![jj as f64; 2]);
            let res = ds.cost(&ii, &jj);
            assert!(
                approx_eq!(f64, exp, res),
                "Test cost for ({},{}): Expect: {} | Result: {}",
                ii,
                jj,
                exp,
                res
            );
        }
    }
}
