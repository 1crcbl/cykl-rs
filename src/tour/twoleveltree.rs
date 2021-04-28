use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::node::{Container, Node};

use super::{Tour, Vertex};

type RcVertex = Rc<RefCell<TltVertex>>;
type WeakVertex = Weak<RefCell<TltVertex>>;
type RcParent = Rc<RefCell<ParentVertex>>;
type WeakParent = Weak<RefCell<ParentVertex>>;

pub type TourOrder = Vec<usize>;

#[derive(Debug)]
pub struct TwoLevelTree<'a> {
    container: &'a Container,
    vertices: Vec<RcVertex>,
    parents: Vec<RcParent>,
}

impl<'a> TwoLevelTree<'a> {
    pub fn new(container: &'a Container, max_groupsize: usize) -> Self {
        let n_parents = (container.len() / max_groupsize) + 1;

        let vertices = container
            .into_iter()
            .map(|n| TltVertex::new(n).to_rc())
            .collect();

        let parents = (0..n_parents)
            .into_iter()
            .map(|id| ParentVertex::new(id, max_groupsize).to_rc())
            .collect();

        Self {
            container,
            vertices,
            parents,
        }
    }

    pub fn init(&mut self, tour: Option<&TourOrder>) {
        let tour = match tour {
            // TODO: is there a better way that can avoid clone?
            Some(t) => t.clone(),
            None => (0..self.vertices.len()).collect(),
        };

        let p_len = self.parents.len();
        let v_len = self.vertices.len();
        for ip in 0..p_len {
            let p = self.parents.get(ip).unwrap();
            let next_p = self.parents.get((ip + 1) % p_len).unwrap();
            let prev_p = if ip == 0 {
                self.parents.last().unwrap()
            } else {
                self.parents.get(ip - 1).unwrap()
            };

            p.borrow_mut().next = Some(Rc::downgrade(next_p));
            next_p.borrow_mut().prev = Some(Rc::downgrade(p));
            p.borrow_mut().prev = Some(Rc::downgrade(prev_p));
            prev_p.borrow_mut().next = Some(Rc::downgrade(p));

            let beg_seg = ip * p.borrow().max_size;
            let end_seg = (beg_seg + p.borrow().max_size).min(v_len);

            for iv in beg_seg..end_seg {
                let v = self.vertices.get(tour[iv]).unwrap();
                v.borrow_mut().parent = Some(Rc::downgrade(p));

                if iv == beg_seg {
                    p.borrow_mut().first = Some(Rc::downgrade(v));
                }

                if iv == end_seg - 1 {
                    p.borrow_mut().last = Some(Rc::downgrade(v));
                }

                let next_v = self.vertices.get(tour[(iv + 1) % v_len]).unwrap();
                let prev_v = if iv == 0 {
                    self.vertices.last().unwrap()
                } else {
                    self.vertices.get(tour[iv - 1]).unwrap()
                };

                v.borrow_mut().next = Some(Rc::downgrade(next_v));
                next_v.borrow_mut().prev = Some(Rc::downgrade(v));
                v.borrow_mut().prev = Some(Rc::downgrade(prev_v));
                prev_v.borrow_mut().next = Some(Rc::downgrade(v));

                p.borrow_mut().size += 1;
            }
        }
    }
}

impl<'a> Tour for TwoLevelTree<'a> {
    type Output = TltVertex;

    #[inline]
    fn get(&self, node_idx: usize) -> Option<&Self::Output> {
        // TODO: check out-of-bound index
        if let Some(v) = self.vertices.get(node_idx) {
            unsafe {
                return v.as_ref().as_ptr().as_ref();
            }
        }

        None
    }

    fn next(&self, _node_idx: usize) -> Option<&Self::Output> {
        todo!()
    }

    fn prev(&self, _node_idx: usize) -> Option<&Self::Output> {
        todo!()
    }

    fn between(&self, _from_idx: usize, _mid_idx: usize, _to_idx: usize) -> bool {
        todo!()
    }

    fn flip(&mut self, _from_idx1: usize, _to_idx1: usize, _from_idx2: usize, _to_idx2: usize) {
        todo!()
    }
}

#[derive(Debug)]
pub struct TltVertex {
    node: Node,
    prev: Option<WeakVertex>,
    next: Option<WeakVertex>,
    parent: Option<WeakParent>,
}

impl TltVertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
            prev: None,
            next: None,
            parent: None,
        }
    }

    pub fn to_rc(self) -> RcVertex {
        Rc::new(RefCell::new(self))
    }
}

impl Vertex for TltVertex {}

impl PartialEq for TltVertex {
    fn eq(&self, other: &Self) -> bool {
        // TODO: expand comparison to pointer.
        self.node == other.node
    }
}

#[derive(Debug)]
struct ParentVertex {
    id: usize,
    size: usize,
    max_size: usize,
    reverse: bool,
    prev: Option<WeakParent>,
    next: Option<WeakParent>,
    first: Option<WeakVertex>,
    last: Option<WeakVertex>,
}

impl ParentVertex {
    fn new(id: usize, max_size: usize) -> Self {
        Self {
            id,
            size: 0,
            max_size,
            reverse: false,
            prev: None,
            next: None,
            first: None,
            last: None,
        }
    }

    fn to_rc(self) -> RcParent {
        Rc::new(RefCell::new(self))
    }
}

impl PartialEq for ParentVertex {
    fn eq(&self, other: &Self) -> bool {
        // TODO: expand comparison to pointer.
        self.id == other.id && self.size == other.size && self.max_size == other.max_size
    }
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::super::tests::create_container;
    use super::*;

    #[test]
    fn test_init_tree() {
        let n_nodes = 10;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);
        tree.init(None);

        assert_eq!(4, tree.parents.len());
        assert_eq!(n_nodes, tree.vertices.len());

        // First parent group
        let first_p = tree.parents[0].borrow();
        assert_eq!(3, first_p.size);
        assert!(first_p.first.is_some());
        assert!(first_p.last.is_some());
        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices[0]),
            first_p.first.as_ref().unwrap()
        ));
        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices[2]),
            first_p.last.as_ref().unwrap()
        ));

        // Last parent group
        let last_p = tree.parents.last().unwrap().borrow();
        assert_eq!(1, last_p.size);
        assert!(last_p.first.is_some());
        assert!(last_p.last.is_some());
        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices.last().unwrap()),
            last_p.first.as_ref().unwrap()
        ));
        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices.last().unwrap()),
            last_p.last.as_ref().unwrap()
        ));

        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.parents.last().unwrap()),
            tree.parents
                .first()
                .unwrap()
                .borrow()
                .prev
                .as_ref()
                .unwrap()
        ));

        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.parents.first().unwrap()),
            tree.parents.last().unwrap().borrow().next.as_ref().unwrap()
        ));
    }
}
