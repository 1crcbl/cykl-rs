use std::{cell::RefCell, rc::Rc};

use crate::{
    metric::{Metric, MetricKind, RcMetric},
    Scalar,
};

type RcNode = Rc<RefCell<InnerNode>>;

#[derive(Debug)]
pub struct Container {
    nodes: Vec<Node>,
    metric: RcMetric,
}

impl Container {
    pub fn new(kind: MetricKind) -> Self {
        Self {
            nodes: Vec::new(),
            metric: Metric::new_as_rc(kind),
        }
    }

    pub fn with_capacity(capacity: usize, kind: MetricKind) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            metric: Metric::new_as_rc(kind),
        }
    }

    /// Adds a new node to the container.
    pub fn add(&mut self, x: Scalar, y: Scalar, z: Scalar) {
        let node = Node::new(self.nodes.len(), x, y, z);
        self.nodes.push(node);
    }

    pub fn get(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }

    /// Calculates and returns the distance between `node1` and `node2`.
    pub fn distance(&self, a: &Node, b: &Node) -> Scalar {
        // TODO: check whether a node with index belongs to this container.
        self.metric.borrow_mut().apply(a, b)
    }

    /// Returns the number of nodes in the container.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl IntoIterator for Container {
    type Item = Node;
    type IntoIter = std::vec::IntoIter<Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'s> IntoIterator for &'s Container {
    type Item = &'s Node;
    type IntoIter = std::slice::Iter<'s, Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    inner: RcNode,
}

#[derive(Debug)]
struct InnerNode {
    index: usize,
    x: Scalar,
    y: Scalar,
    z: Scalar,
}

impl Node {
    pub fn new(index: usize, x: Scalar, y: Scalar, z: Scalar) -> Self {
        let inner = InnerNode { index, x, y, z };

        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.inner.borrow().index
    }

    #[inline]
    pub fn x(&self) -> Scalar {
        self.inner.borrow().x
    }

    #[inline]
    pub fn y(&self) -> Scalar {
        self.inner.borrow().y
    }

    #[inline]
    pub fn z(&self) -> Scalar {
        self.inner.borrow().z
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
            && self.x() == self.x()
            && self.y() == self.y()
            && self.z() == self.z()
    }
}
