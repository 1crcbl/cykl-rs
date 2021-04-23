use crate::node::{Container};

use super::{Tour, Vertex};

///
/// Vertex[Tracker[ii]] = n_ii
/// Initially:
/// Index:    | 0   | 1   | 2   | 3   | 4   | 5   |
/// Vertex:   | n_0 | n_1 | n_2 | n_3 | n_4 | n_5 |
/// Tracker:  | 0   | 1   | 2   | 3   | 4   | 5   |
///
/// After some operations:
/// Index:    | 0   | 1   | 2   | 3   | 4   | 5   |
/// Vertex:   | n_4 | n_2 | n_1 | n_5 | n_0 | n_3 |
/// Tracker:  | 4   | 2   | 1   | 5   | 0   | 3   |
#[derive(Debug)]
pub struct ArrayTour<'a> {
    container: &'a Container,
    vertices: Vec<Vertex>,
    tracker: Vec<usize>,
}

impl<'a> ArrayTour<'a> {
    pub fn new(container: &'a Container) -> Self {
        let vertices: Vec<Vertex> = container.into_iter().map(|n| Vertex::new(n)).collect();
        let tracker = (0..vertices.len()).collect();

        Self {
            container,
            vertices,
            tracker,
        }
    }
}

impl<'a> Tour for ArrayTour<'a> {
    fn next(&self, node_idx: usize) -> Option<&Vertex> {
        if node_idx == self.vertices.len() - 1{
            return self.vertices.first();
        } else if node_idx < self.vertices.len() - 1 {
            return self.vertices.get(*self.tracker.get(node_idx).unwrap());
        }

        None
    }

    fn prev(&self, node_idx: usize) -> Option<&Vertex> {
        if node_idx == 0 {
            return self.vertices.last();
        } else if node_idx < self.vertices.len() - 1 {
            return self.vertices.get(*self.tracker.get(node_idx).unwrap());
        }

        None
    }

    fn between(&self, _from_idx: usize, _mid_idx: usize, _to_idx: usize) -> bool {
        todo!()
    }

    fn flip(&self, _from_idx1: usize, _to_idx1: usize, _from_idx2: usize, _to_idx2: usize) {
        todo!()
    }
}

#[allow(dead_code)]
mod tests {
    use super::*;

    use crate::node::Container;
    use crate::metric::MetricKind;

    fn create_container() -> Container {
        let mut container = Container::new(MetricKind::Euc2d);
        container.add(1., 2., 3.);      // 0
        container.add(4., 5., 6.);      // 1
        container.add(7., 8., 9.);      // 2
        container.add(10., 11., 12.);   // 3
        container.add(13., 14., 15.);   // 4
        container
    }
}