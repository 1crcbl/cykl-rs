use std::{collections::HashMap, fmt, ptr::NonNull, rc::Rc};

use tspf::{Tsp, WeightKind};

use crate::Scalar;

#[derive(Clone)]
pub struct Repo {
    inner: Option<NonNull<InnerRepo>>,
}

#[derive(Debug)]
struct InnerRepo {
    // Single depot for now.
    depot: Option<DataNode>,
    nodes: Vec<DataNode>,
    cache: HashMap<(usize, usize), Scalar>,
    kind: WeightKind,
}

impl Repo {
    /// Adds a new node to the container.
    #[inline]
    pub fn add(&mut self, kind: NodeKind, pos: Vec<Scalar>) {
        match &self.inner {
            Some(inner) => unsafe {
                let ptr = inner.as_ptr();
                let node = DataNode::new((*ptr).nodes.len(), kind, pos);
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
                        let d = (*ptr).kind.cost(a.pos(), b.pos());
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
    pub fn dist_lb(&self, _a: &DataNode, _b: &DataNode) -> Scalar {
        todo!()
        // if a.index() == b.index() {
        //     return 0.;
        // }

        // match self.inner {
        //     Some(inner) => unsafe { (*inner.as_ptr()).func_lb.as_ref()(a, b) },
        //     None => panic!("Nullpointer"),
        // }
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

impl From<Tsp> for Repo {
    fn from(mut tsp: Tsp) -> Self {
        let mut nodes = Vec::with_capacity(tsp.dim());
        let nc = std::mem::take(tsp.node_coords_mut());
        for (idx, pt) in nc {
            let kind = if tsp.depots().contains(&idx) {
                NodeKind::Depot
            } else {
                NodeKind::Customer
            };

            let (_, pos) = pt.into_value();

            nodes.push(DataNode::new(idx, kind, pos));
        }

        let inner = Box::new(InnerRepo {
            depot: None,
            nodes,
            cache: HashMap::new(),
            kind: tsp.weight_kind(),
        });

        Self {
            inner: Some(Box::leak(inner).into()),
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
    met_kind: WeightKind,
    capacity: Option<usize>,
    costs: Option<Vec<Vec<Scalar>>>,
    mat_kind: Option<MatrixKind>,
}

impl RepoBuilder {
    pub fn new(kind: WeightKind) -> Self {
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
                    .map(|id| DataNode::new(id, NodeKind::Customer, Vec::with_capacity(0)))
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

        let inner = Box::new(InnerRepo {
            depot: None,
            nodes,
            cache: hm,
            kind: self.met_kind,
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
    // Note: might become Arc<RwLock> later.
    inner: Rc<InnerNode>,
}

#[derive(Clone, Debug)]
struct InnerNode {
    index: usize,
    kind: NodeKind,
    pos: Vec<Scalar>,
    /// Demand
    d: Scalar,
}

impl DataNode {
    pub fn new(index: usize, kind: NodeKind, pos: Vec<Scalar>) -> Self {
        Self::with_demand(index, kind, pos, 0.)
    }

    pub fn with_demand(index: usize, kind: NodeKind, pos: Vec<Scalar>, d: Scalar) -> Self {
        Self {
            inner: Rc::new(InnerNode {
                index,
                kind,
                pos,
                d,
            }),
        }
    }

    /// Returns the index of a node.
    #[inline]
    pub fn index(&self) -> usize {
        self.inner.index
    }

    #[inline]
    pub fn pos(&self) -> &Vec<Scalar> {
        &self.inner.pos
    }
}

// impl PartialOrd for DataNode {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         todo!()
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Depot,
    Customer,
}
