use std::{f64::consts::PI, ptr::NonNull};

const EARTH_RADIUS: f64 = 6378.388;

pub trait GetIndex {
    /// Returns a node's index.
    fn get(&self) -> usize;
}

impl GetIndex for usize {
    fn get(&self) -> usize {
        *self
    }
}

impl GetIndex for NodeIndex {
    fn get(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeIndex {
    index: usize,
    kind: NodeKind,
}

impl NodeIndex {
    pub(crate) fn new(index: usize, kind: NodeKind) -> Self {
        Self { index, kind }
    }

    /// Returns the location's index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the location's kind.
    pub fn kind(&self) -> NodeKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    Depot,
    Target,
}

#[derive(Clone, Copy, Debug)]
pub struct DataStore<M> {
    inner: Option<NonNull<InnerStore<M>>>,
}

struct InnerStore<M> {
    dim: usize,
    metric: Metric,
    nodes: Vec<NodeIndex>,
    meta: Vec<M>,
    coords: Vec<f64>,
    // Compute and store all cost in a big vec.
    // This simplifies implementation and interface but comes with huge cost for memory,
    // especially when we also need to save extra things for alpha-nearness scheme.
    // Need optimisation for large problems, but later.
    costs: Vec<f64>,
}

impl<M> DataStore<M> {
    pub fn new(metric: Metric) -> Self {
        let inner = InnerStore {
            dim: metric.dim(),
            metric,
            nodes: Vec::new(),
            meta: Vec::new(),
            coords: Vec::new(),
            costs: Vec::with_capacity(0),
        };

        Self {
            inner: NonNull::new(Box::leak(Box::new(inner))),
        }
    }

    pub fn with_capacity(metric: Metric, capacity: usize) -> Self {
        let inner = InnerStore {
            dim: metric.dim(),
            metric,
            nodes: Vec::with_capacity(capacity),
            meta: Vec::with_capacity(capacity),
            coords: Vec::with_capacity(capacity * metric.dim()),
            costs: Vec::with_capacity(0),
        };

        Self {
            inner: NonNull::new(Box::leak(Box::new(inner))),
        }
    }

    /// Returns the number of nodes registered in the store.
    #[inline]
    pub fn len(&self) -> usize {
        match self.inner {
            Some(inner) => unsafe { inner.as_ref().nodes.len() },
            None => 0,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self.inner {
            Some(inner) => unsafe { inner.as_ref().nodes.is_empty() },
            None => true,
        }
    }

    #[inline]
    pub fn add(&mut self, kind: NodeKind, mut pos: Vec<f64>, meta: M) -> Option<NodeIndex> {
        self.inner.and_then(|inner| unsafe {
            if pos.len() != inner.as_ref().dim {
                panic!("Len mismatched")
            }

            let idx = inner.as_ref().nodes.len();
            let node = NodeIndex::new(idx, kind);

            (*inner.as_ptr()).nodes.push(node);
            (*inner.as_ptr()).meta.push(meta);
            (*inner.as_ptr()).coords.append(&mut pos);

            inner.as_ref().nodes.get(idx).cloned()
        })
    }

    #[inline]
    pub fn cost<I>(&self, a: &I, b: &I) -> f64
    where
        I: GetIndex + PartialEq + Eq,
    {
        if a == b {
            0.
        } else {
            match self.inner {
                Some(inner) => unsafe {
                    inner.as_ref().costs[a.get() * inner.as_ref().nodes.len() + b.get()]
                },
                None => 0.,
            }
        }
    }

    pub fn compute(&mut self) {
        if let Some(inner) = self.inner {
            unsafe {
                let n_nodes = inner.as_ref().nodes.len();
                let dim = inner.as_ref().dim;

                let mut result = vec![0.; n_nodes * n_nodes];
                inner
                    .as_ref()
                    .coords
                    .chunks(dim)
                    .enumerate()
                    .for_each(|(idx1, x1)| {
                        let tmp = idx1 * n_nodes;
                        inner
                            .as_ref()
                            .coords
                            .chunks(dim)
                            .enumerate()
                            .for_each(|(idx2, x2)| {
                                let pos = tmp + idx2;
                                if idx1 != idx2 {
                                    result[pos] = inner.as_ref().metric.cost(x1, x2);
                                }
                            })
                    });

                (*inner.as_ptr()).costs = result;
            }
        }
    }
}

impl<'s, M> IntoIterator for &'s DataStore<M> {
    type Item = &'s NodeIndex;

    type IntoIter = std::slice::Iter<'s, NodeIndex>;

    fn into_iter(self) -> Self::IntoIter {
        match self.inner {
            Some(inner) => unsafe { inner.as_ref().nodes.iter() },
            None => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Metric {
    /// Weights are explicitly given in the data file.
    Explicit,
    /// Two-dimensional Euclidean distance.
    Euc2d,
    /// Three-dimensional Euclidean distance.
    Euc3d,
    /// Two-dimensional maximum distance.
    Max2d,
    /// Three-dimensional maximum distance.
    Max3d,
    /// Two-dimensional Manhattan distance.
    Man2d,
    /// Three-dimensional Manhattan distance.
    Man3d,
    /// Rounded-up two dimensional Euclidean distance.
    Ceil2d,
    /// Geographical distance.
    Geo,
    /// Special distance function for problems ```att48``` and ```att532```.
    Att,
    /// Special distance function for crystallography problems of version 1.
    Xray1,
    /// Special distance function for crystallography problems of version 2.
    Xray2,
    /// Distance function defined by users.
    Custom,
    /// No distance function is given.
    Undefined,
}

impl Metric {
    pub fn dim(&self) -> usize {
        match self {
            Metric::Explicit => 0,
            Metric::Euc2d
            | Metric::Max2d
            | Metric::Man2d
            | Metric::Ceil2d
            | Metric::Geo
            | Metric::Att => 2,
            Metric::Euc3d | Metric::Max3d | Metric::Man3d | Metric::Xray1 | Metric::Xray2 => 3,
            Metric::Custom => todo!(),
            Metric::Undefined => 0,
        }
    }

    pub fn cost(&self, a: &[f64], b: &[f64]) -> f64 {
        match self {
            Self::Euc2d => euc_2d(a, b),
            Self::Euc3d => euc_3d(a, b),
            Self::Geo => geo(a, b),
            Self::Max2d => max_2d(a, b),
            Self::Max3d => max_3d(a, b),
            Self::Man2d => man_2d(a, b),
            Self::Man3d => man_3d(a, b),
            Self::Ceil2d => euc_2d(a, b).round(),
            Self::Att => att(a, b),
            Self::Xray1 => xray1(a, b),
            Self::Xray2 => xray2(a, b),
            _ => 0.,
        }
    }
}

/// Calculates the 2D-Euclidean distance between two points.
#[inline]
pub fn euc_2d(a: &[f64], b: &[f64]) -> f64 {
    euc(a, b, 2)
}

/// Calculates the 3D-Euclidean distance between two points.
#[inline]
pub fn euc_3d(a: &[f64], b: &[f64]) -> f64 {
    euc(a, b, 3)
}

#[inline]
fn euc(a: &[f64], b: &[f64], k: usize) -> f64 {
    a.iter()
        .take(k)
        .zip(b.iter().take(k))
        .fold(0_f64, |acc, (x1, x2)| acc + (x1 - x2).powi(2))
        .sqrt()
}

/// Calculates the 2D-Manhattan distance between two points.
#[inline]
pub fn man_2d(a: &[f64], b: &[f64]) -> f64 {
    man(a, b, 2)
}

/// Calculates the 3D-Manhattan distance between two points.
#[inline]
pub fn man_3d(a: &[f64], b: &[f64]) -> f64 {
    man(a, b, 3)
}

#[inline]
fn man(a: &[f64], b: &[f64], k: usize) -> f64 {
    a.iter()
        .take(k)
        .zip(b.iter().take(k))
        .fold(0_f64, |acc, (x1, x2)| acc + (x1 - x2).abs())
}

/// Calculates the 2D maximum distance between two points.
#[inline]
pub fn max_2d(a: &[f64], b: &[f64]) -> f64 {
    max(a, b, 2)
}

/// Calculates the 3D maximum distance between two points.
#[inline]
pub fn max_3d(a: &[f64], b: &[f64]) -> f64 {
    max(a, b, 3)
}

#[inline]
fn max(a: &[f64], b: &[f64], k: usize) -> f64 {
    a.iter()
        .take(k)
        .zip(b.iter().take(k))
        .fold(0_f64, |acc, (x1, x2)| acc.max((x1 - x2).abs()))
}

/// Calculates the geographical between two points.
#[inline]
pub fn geo(a: &[f64], b: &[f64]) -> f64 {
    let (lat_a, lon_a) = (to_geo_coord(a[0]), to_geo_coord(a[1]));
    let (lat_b, lon_b) = (to_geo_coord(b[0]), to_geo_coord(b[1]));

    let q1 = (lon_a - lon_b).cos();
    let q2 = (lat_a - lat_b).cos();
    let q3 = (lat_a + lat_b).cos();
    let q4 = (0.5 * ((1. + q1) * q2 - (1. - q1) * q3)).acos();
    EARTH_RADIUS * q4 + 1.
}

#[inline]
fn to_geo_coord(x: f64) -> f64 {
    let deg = x.trunc();
    let min = x - deg;
    PI * (deg + 5. * min / 3.) / 180.
}

/// Calculates the distance between two points for dataset from AT&T Bell laboratory, published by Padberg and Rinaldi in 1987.
#[inline]
pub fn att(a: &[f64], b: &[f64]) -> f64 {
    (a.iter()
        .take(2)
        .zip(b.iter().take(2))
        .fold(0_f64, |acc, (x1, x2)| acc + (x1 - x2).powi(2))
        / 10.)
        .sqrt()
}

/// Calculates the distance between two points for crystallography problems (version 1).
#[inline]
pub fn xray1(a: &[f64], b: &[f64]) -> f64 {
    let dx = (a[0] - b[0]).abs();
    let pr = dx.min((dx - 360.).abs());
    let dy = (a[1] - b[1]).abs();
    let dz = (a[2] - b[2]).abs();
    100. * pr.max(dy.max(dz))
}

/// Calculates the distance between two points for crystallography problems (version 2).
#[inline]
pub fn xray2(a: &[f64], b: &[f64]) -> f64 {
    let dx = (a[0] - b[0]).abs();
    let pr = dx.min((dx - 360.).abs());
    let dy = (a[1] - b[1]).abs();
    let dz = (a[2] - b[2]).abs();
    100. * (pr / 1.25).max((dy / 1.5).max(dz / 1.15))
}
