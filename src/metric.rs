use std::{cell::RefCell, collections::HashMap, rc::Rc};
use std::fmt;

use crate::node::Node;
use crate::Scalar;

pub (crate) type RcMetric = Rc<RefCell<Metric>>;

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

    pub fn apply(&mut self, node1: &Node, node2: &Node) -> Scalar {
        if node1.index() > node2.index() {
            return self.apply(node2, node1);
        }
        let key = (node1.index(), node2.index());

        match self.cache.get(&key) {
            Some(d) => *d,
            None => {
                let d = self.func.as_ref()(node1, node2);
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
    /// Uses two-dimensional Euclidean distance.
    Euc2d,
    /// Uses three-dimensional Euclidean distance.
    Euc3d,
    ///
    Undefined,
}

fn dist_euc_2d(node1: &Node, node2: &Node) -> Scalar {
    ((node1.x() - node2.x()).powi(2) + (node1.y() - node2.y()).powi(2)).sqrt()
}

fn dist_euc_3d(node1: &Node, node2: &Node) -> Scalar {
    ((node1.x() - node2.x()).powi(2)
        + (node1.y() - node2.y()).powi(2)
        + (node1.z() - node2.z()).powi(2))
    .sqrt()
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::*;

    use crate::node::Node;

    #[test]
    fn test_euc_2d() {
        let (node1, node2) = create_node_pair();
        assert_eq!((34 as Scalar).sqrt(), dist_euc_2d(&node1, &node2));
    }

    #[test]
    fn test_euc_3d() {
        let (node1, node2) = create_node_pair();
        assert_eq!((35 as Scalar).sqrt(), dist_euc_3d(&node1, &node2));
    }

    fn create_node_pair() -> (Node, Node) {
        let node1 = Node::new(0, 1., 2., 3.);
        let node2 = Node::new(1, 6., 5., 4.);
        (node1, node2)
    }
}