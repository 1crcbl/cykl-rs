use std::f64::consts::PI;
use std::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::node::Node;
use crate::Scalar;

const EARTH_RADIUS: Scalar = 6378.388;

pub(crate) type RcMetric = Rc<RefCell<Metric>>;

pub struct Metric {
    cache: HashMap<(usize, usize), Scalar>,
    kind: MetricKind,
    func: Box<dyn Fn(&Node, &Node) -> Scalar>,
}

impl Metric {
    pub fn new(kind: MetricKind) -> Self {
        let f: Box<dyn Fn(&Node, &Node) -> Scalar> = match &kind {
            MetricKind::Euc2d => Box::new(dist_euc_2d),
            MetricKind::Euc3d => Box::new(dist_euc_3d),
            MetricKind::Geo => Box::new(dist_geo),
            _ => unimplemented!(),
        };

        Self {
            cache: HashMap::new(),
            kind: kind,
            func: f,
        }
    }

    pub fn new_as_rc(kind: MetricKind) -> RcMetric {
        let metric = Self::new(kind);
        Rc::new(RefCell::new(metric))
    }

    pub fn apply(&mut self, a: &Node, b: &Node) -> Scalar {
        if a.index() > b.index() {
            return self.apply(b, a);
        }
        let key = (a.index(), b.index());

        match self.cache.get(&key) {
            Some(d) => *d,
            None => {
                let d = self.func.as_ref()(a, b);
                self.cache.insert(key, d);
                d
            }
        }
    }
}

impl fmt::Debug for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metric")
            .field("kind", &self.kind)
            .field("cache", &self.cache)
            .finish()
    }
}

#[derive(Debug)]
pub enum MetricKind {
    /// Two-dimensional Euclidean distance.
    Euc2d,
    /// Three-dimensional Euclidean distance.
    Euc3d,
    /// Geographical distance.
    Geo,
    ///
    Undefined,
}

fn dist_euc_2d(a: &Node, b: &Node) -> Scalar {
    ((a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2)).sqrt()
}

fn dist_euc_3d(a: &Node, b: &Node) -> Scalar {
    ((a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2) + (a.z() - b.z()).powi(2)).sqrt()
}

fn dist_geo(a: &Node, b: &Node) -> Scalar {
    let (lat_a, lon_a) = (to_geo_coord(a.x()), to_geo_coord(a.y()));
    let (lat_b, lon_b) = (to_geo_coord(b.x()), to_geo_coord(b.y()));

    let q1 = (lon_a - lon_b).cos();
    let q2 = (lat_a - lat_b).cos();
    let q3 = (lat_a + lat_b).cos();
    let q4 = (0.5 * ((1. + q1) * q2 - (1. - q1) * q3)).acos();
    EARTH_RADIUS * q4 + 1.
}

fn to_geo_coord(x: Scalar) -> Scalar {
    let deg = x.round();
    let min = x - deg;
    PI * (deg + 5. * min / 3.) / 180.
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::*;

    use crate::node::Node;

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

    fn create_node_pair() -> (Node, Node) {
        let a = Node::new(0, 1., 2., 3.);
        let b = Node::new(1, 6., 5., 4.);
        (a, b)
    }
}
