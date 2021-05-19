use std::{cell::RefCell, collections::HashMap, f64::consts::PI, fmt, ptr::NonNull, rc::Rc};

use crate::Scalar;

const EARTH_RADIUS: Scalar = 6378.388;

#[derive(Clone)]
pub struct Repo {
    inner: Option<NonNull<InnerRepo>>,
}

struct InnerRepo {
    nodes: Vec<DataNode>,
    cache: HashMap<(usize, usize), Scalar>,
    kind: MetricKind,
    func: Box<dyn Fn(&DataNode, &DataNode) -> Scalar>,
    func_lb: Box<dyn Fn(&DataNode, &DataNode) -> Scalar>,
}

impl Repo {
    /// Adds a new node to the container.
    #[inline]
    pub fn add(&mut self, x: Scalar, y: Scalar, z: Scalar) {
        match &self.inner {
            Some(inner) => unsafe {
                let ptr = inner.as_ptr();
                let node = DataNode::new((*ptr).nodes.len(), x, y, z);
                (*ptr).nodes.push(node);
            },
            None => panic!("Nullpointer"),
        };
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&DataNode> {
        match &self.inner {
            Some(inner) => unsafe { (*inner.as_ptr()).nodes.get(index) },
            None => panic!("Nullpointer"),
        }
    }

    /// Calculates the distance between two nodes.
    #[inline]
    pub fn distance(&self, a: &DataNode, b: &DataNode) -> Scalar {
        // TODO: check whether a node with index belongs to this container.
        if a.index() == b.index() {
            return 0.;
        }

        let key = if a.index() > b.index() {
            (b.index(), a.index())
        } else {
            (a.index(), b.index())
        };

        match self.inner {
            Some(inner) => unsafe {
                let ptr = inner.as_ptr();
                let val = (*ptr).cache.get(&key);
                match val {
                    Some(d) => *d,
                    None => {
                        let d = (*ptr).func.as_ref()(a, b);
                        (*ptr).cache.insert(key, d);
                        d
                    }
                }
            },
            None => panic!("Nullpointer"),
        }
    }

    /// Calculates the distance between two nodes at the given indices.
    #[inline]
    pub fn distance_at(&self, index_a: usize, index_b: usize) -> Scalar {
        if index_a == index_b {
            return 0.;
        }

        match (self.get(index_a), self.get(index_b)) {
            (Some(a), Some(b)) => self.distance(a, b),
            _ => 0.,
        }
    }

    /// Calculates the lower bound of the distance between two nodes.
    #[inline]
    pub fn dist_lb(&self, a: &DataNode, b: &DataNode) -> Scalar {
        if a.index() == b.index() {
            return 0.;
        }

        match self.inner {
            Some(inner) => unsafe { (*inner.as_ptr()).func_lb.as_ref()(a, b) },
            None => panic!("Nullpointer"),
        }
    }

    /// Calculates the lower bound of the distance between two nodes at the given indices.
    #[inline]
    pub fn dist_lb_at(&self, index_a: usize, index_b: usize) -> Scalar {
        if index_a == index_b {
            return 0.;
        }

        match (self.get(index_a), self.get(index_b)) {
            (Some(a), Some(b)) => self.dist_lb(a, b),
            _ => 0.,
        }
    }

    /// Returns the number of nodes in the container.
    #[inline]
    pub fn size(&self) -> usize {
        match &self.inner {
            Some(inner) => unsafe { (*inner.as_ptr()).nodes.len() },
            None => panic!("Nullpointer"),
        }
    }
}

impl fmt::Debug for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            Some(inner) => unsafe {
                let ptr = inner.as_ptr();
                f.debug_struct("Repo")
                    .field("kind", &(*ptr).kind)
                    .field("cache", &(*ptr).cache)
                    .finish()
            },
            None => f.debug_struct("Repo: null").finish(),
        }
    }
}

impl<'s> IntoIterator for &'s Repo {
    type Item = &'s DataNode;
    type IntoIter = std::slice::Iter<'s, DataNode>;

    fn into_iter(self) -> Self::IntoIter {
        match &self.inner {
            Some(inner) => unsafe { (*inner.as_ptr()).nodes.iter() },
            None => panic!("Nullpointer"),
        }
    }
}

#[derive(Debug)]
pub struct RepoBuilder {
    met_kind: MetricKind,
    capacity: Option<usize>,
    costs: Option<Vec<Vec<Scalar>>>,
    mat_kind: Option<MatrixKind>,
}

impl RepoBuilder {
    pub fn new(kind: MetricKind) -> Self {
        Self {
            met_kind: kind,
            capacity: None,
            costs: None,
            mat_kind: None,
        }
    }

    pub fn build(self) -> Repo {
        let (hm, nodes) = match (&self.mat_kind, &self.costs) {
            (Some(kind), Some(costs)) => {
                let mut hm = HashMap::new();
                let n_nodes = costs.len();
                let nodes: Vec<DataNode> = (0..n_nodes)
                    .map(|id| DataNode::new(id, 0., 0., 0.))
                    .collect();

                let make_key_pair = match kind {
                    MatrixKind::Full => |row: usize, col: usize| -> (usize, usize) { (row, col) },
                    MatrixKind::Upper => |row: usize, col: usize| -> (usize, usize) {
                        let new_col = col + row;
                        (row, new_col)
                    },
                    MatrixKind::Lower => |row: usize, col: usize| -> (usize, usize) { (col, row) },
                };

                for (ridx, row) in costs.iter().enumerate() {
                    for (cidx, val) in row.iter().enumerate() {
                        let key = make_key_pair(ridx, cidx);
                        if key.0 < key.1 {
                            hm.insert(key, *val);
                        }
                    }
                }

                (hm, Some(nodes))
            }
            _ => (HashMap::new(), None),
        };

        let nodes = match (&self.capacity, nodes) {
            (Some(cap), Some(mut tmp_nodes)) => {
                let mut v = Vec::with_capacity(std::cmp::max(*cap, tmp_nodes.len()));
                v.append(&mut tmp_nodes);
                v
            }
            (Some(cap), None) => Vec::with_capacity(*cap),
            (None, Some(tmp_nodes)) => tmp_nodes,
            (None, None) => Vec::new(),
        };

        let (func, func_lb): (
            Box<dyn Fn(&DataNode, &DataNode) -> Scalar>,
            Box<dyn Fn(&DataNode, &DataNode) -> Scalar>,
        ) = match &self.met_kind {
            MetricKind::Euc2d => (Box::new(euc_2d), Box::new(euc_2d_lb)),
            MetricKind::Euc3d => (Box::new(euc_3d), Box::new(euc_3d_lb)),
            MetricKind::Geo => (Box::new(geo), Box::new(geo_lb)),
            _ => unimplemented!(),
        };

        let inner = Box::new(InnerRepo {
            nodes,
            cache: hm,
            kind: self.met_kind,
            func,
            func_lb,
        });

        Repo {
            inner: Some(Box::leak(inner).into()),
        }
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn costs(mut self, costs: Vec<Vec<Scalar>>, kind: MatrixKind) -> Self {
        self.costs = Some(costs);
        self.mat_kind = Some(kind);
        self
    }
}

#[derive(Debug)]
pub enum MatrixKind {
    Full,
    Upper,
    Lower,
}

#[derive(Clone, Debug)]
pub struct DataNode {
    inner: Rc<RefCell<InnerNode>>,
}

#[derive(Debug)]
struct InnerNode {
    index: usize,
    x: Scalar,
    y: Scalar,
    z: Scalar,
    /// Weight.
    w: Scalar,
}

impl DataNode {
    pub fn new(index: usize, x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self::with_weight(index, x, y, z, 0.)
    }

    pub fn with_weight(index: usize, x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> Self {
        let inner = InnerNode { index, x, y, z, w };

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

    #[inline]
    pub fn w(&self) -> Scalar {
        self.inner.borrow().w
    }

    #[inline]
    pub fn set_w(&mut self, w: Scalar) {
        self.inner.borrow_mut().w = w;
    }
}

impl PartialEq for DataNode {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
            && self.x() == self.x()
            && self.y() == self.y()
            && self.z() == self.z()
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

/// Returns the 2D-Euclidean distance between two nodes.
#[inline]
pub(super) fn euc_2d(a: &DataNode, b: &DataNode) -> Scalar {
    ((a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2)).sqrt()
}

/// Returns the lower bound of 2D-Euclidean distance between two nodes.
#[inline]
pub(super) fn euc_2d_lb(a: &DataNode, b: &DataNode) -> Scalar {
    let (dx, dy) = ((a.x() - b.x()).abs(), (a.y() - b.y()).abs());
    dx.max(dy)
}

/// Returns the 3D-Euclidean distance between two nodes.
#[inline]
pub(super) fn euc_3d(a: &DataNode, b: &DataNode) -> Scalar {
    ((a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2) + (a.z() - b.z()).powi(2)).sqrt()
}

/// Returns the lower bound of the 3D-Euclidean distance between two nodes.
#[inline]
pub(super) fn euc_3d_lb(a: &DataNode, b: &DataNode) -> Scalar {
    let (dx, dy, dz) = (
        (a.x() - b.x()).abs(),
        (a.y() - b.y()).abs(),
        (a.z() - b.z()).abs(),
    );
    dx.max(dy).max(dz)
}

#[inline]
pub(super) fn geo(a: &DataNode, b: &DataNode) -> Scalar {
    let (lat_a, lon_a) = (to_geo_coord(a.x()), to_geo_coord(a.y()));
    let (lat_b, lon_b) = (to_geo_coord(b.x()), to_geo_coord(b.y()));

    let q1 = (lon_a - lon_b).cos();
    let q2 = (lat_a - lat_b).cos();
    let q3 = (lat_a + lat_b).cos();
    let q4 = (0.5 * ((1. + q1) * q2 - (1. - q1) * q3)).acos();
    EARTH_RADIUS * q4 + 1.
}

#[inline]
pub(super) fn geo_lb(_a: &DataNode, _b: &DataNode) -> Scalar {
    todo!()
}

#[allow(dead_code)]
fn to_geo_coord(x: Scalar) -> Scalar {
    let deg = x.round();
    let min = x - deg;
    PI * (deg + 5. * min / 3.) / 180.
}
