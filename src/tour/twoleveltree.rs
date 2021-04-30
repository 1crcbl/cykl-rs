use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::node::{Container, Node};

use super::{between, Tour, TourOrder, Vertex};

type RcVertex = Rc<RefCell<TltVertex>>;
type WeakVertex = Weak<RefCell<TltVertex>>;
type RcParent = Rc<RefCell<ParentVertex>>;
type WeakParent = Weak<RefCell<ParentVertex>>;

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
}

impl<'a> Tour for TwoLevelTree<'a> {
    type TourNode = TltVertex;

    fn init(&mut self, tour: Option<&TourOrder>) {
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

            p.borrow_mut().next = Rc::downgrade(next_p);
            next_p.borrow_mut().prev = Rc::downgrade(p);
            p.borrow_mut().prev = Rc::downgrade(prev_p);
            prev_p.borrow_mut().next = Rc::downgrade(p);

            let beg_seg = ip * p.borrow().max_size;
            let end_seg = (beg_seg + p.borrow().max_size).min(v_len);

            for iv in beg_seg..end_seg {
                let v = self.vertices.get(tour[iv]).unwrap();
                v.borrow_mut().seq_id = iv - beg_seg;
                v.borrow_mut().parent = Rc::downgrade(p);

                if iv == beg_seg {
                    p.borrow_mut().first = Rc::downgrade(v);
                }

                if iv == end_seg - 1 {
                    p.borrow_mut().last = Rc::downgrade(v);
                }

                let next_v = self.vertices.get(tour[(iv + 1) % v_len]).unwrap();
                let prev_v = if iv == 0 {
                    self.vertices.last().unwrap()
                } else {
                    self.vertices.get(tour[iv - 1]).unwrap()
                };

                v.borrow_mut().next = Rc::downgrade(next_v);
                next_v.borrow_mut().prev = Rc::downgrade(v);
                v.borrow_mut().prev = Rc::downgrade(prev_v);
                prev_v.borrow_mut().next = Rc::downgrade(v);

                p.borrow_mut().size += 1;
            }
        }
    }

    /// The operation should compute in *O*(1) time.
    #[inline]
    fn get(&self, node_idx: usize) -> Option<&Self::TourNode> {
        if let Some(v) = self.vertices.get(node_idx) {
            unsafe {
                return v.as_ref().as_ptr().as_ref();
            }
        }

        None
    }

    /// The operation should compute in *O*(1) time.
    // Note: There might be hit in performance due to memory safeguarding. Need benchmark to verify.
    #[inline]
    fn next(&self, node_idx: usize) -> Option<&Self::TourNode> {
        if let Some(v) = self.vertices.get(node_idx) {
            let borrow_v = v.borrow();
            if let Some(p) = borrow_v.parent.upgrade() {
                let kin = if p.borrow().reverse {
                    &borrow_v.prev
                } else {
                    &borrow_v.next
                };

                return match kin.upgrade() {
                    Some(node) => unsafe { node.as_ref().as_ptr().as_ref() },
                    None => None,
                };
            }
        }

        None
    }

    /// The operation should compute in *O*(1) time.
    // Note: There might be hit in performance due to memory safeguarding. Need benchmark to verify.
    #[inline]
    fn prev(&self, node_idx: usize) -> Option<&Self::TourNode> {
        if let Some(v) = self.vertices.get(node_idx) {
            let borrow_v = v.borrow();
            if let Some(p) = borrow_v.parent.upgrade() {
                let kin = if p.borrow().reverse {
                    &borrow_v.next
                } else {
                    &borrow_v.prev
                };

                return match kin.upgrade() {
                    Some(node) => unsafe { node.as_ref().as_ptr().as_ref() },
                    None => None,
                };
            }
        }

        None
    }

    /// This implementation should compute in *O*(1) time, with some constants.
    fn between(&self, from_idx: usize, mid_idx: usize, to_idx: usize) -> bool {
        match (self.get(from_idx), self.get(mid_idx), self.get(to_idx)) {
            (Some(from), Some(mid), Some(to)) => {
                let (fp, mp, tp) = (&from.parent, &mid.parent, &to.parent);
                match (
                    Weak::ptr_eq(fp, mp),
                    Weak::ptr_eq(mp, tp),
                    Weak::ptr_eq(tp, fp),
                ) {
                    (true, true, true) => between(from.seq_id, mid.seq_id, to.seq_id),
                    (true, false, false) => {
                        if let Some(p) = fp.upgrade() {
                            p.borrow().reverse ^ (from.seq_id <= mid.seq_id)
                        } else {
                            false
                        }
                    }
                    (false, true, false) => {
                        if let Some(p) = mp.upgrade() {
                            p.borrow().reverse ^ (mid.seq_id <= to.seq_id)
                        } else {
                            false
                        }
                    }
                    (false, false, true) => {
                        if let Some(p) = tp.upgrade() {
                            p.borrow().reverse ^ (to.seq_id <= from.seq_id)
                        } else {
                            false
                        }
                    }
                    (false, false, false) => unsafe {
                        between(
                            (&*fp.as_ptr()).borrow().id,
                            (&*mp.as_ptr()).borrow().id,
                            (&*tp.as_ptr()).borrow().id,
                        )
                    },
                    // (true, true, false)
                    // (true, false, true)
                    // (false, true, true)
                    _ => panic!("The transitivity requirement is violated."),
                }
            }
            _ => false,
        }
    }

    /// This implementation of the `flip` operation takes at least Sigma(N) time to compute.
    fn flip(&mut self, _from_idx1: usize, _to_idx1: usize, _from_idx2: usize, _to_idx2: usize) {
        todo!()
    }
}

#[derive(Debug)]
pub struct TltVertex {
    /// Sequential ID in the parent node.
    ///
    /// If a vertex is not attached to any parent node, `usize::MAX` will be assigned.
    seq_id: usize,
    node: Node,
    visited: bool,
    prev: WeakVertex,
    next: WeakVertex,
    parent: WeakParent,
}

impl TltVertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
            seq_id: usize::MAX,
            visited: false,
            prev: Weak::new(),
            next: Weak::new(),
            parent: Weak::new(),
        }
    }

    fn to_rc(self) -> RcVertex {
        Rc::new(RefCell::new(self))
    }
}

impl Vertex for TltVertex {
    fn is_visited(&self) -> bool {
        self.visited
    }

    fn visited(&mut self, flag: bool) {
        self.visited = flag;
    }
}

impl PartialEq for TltVertex {
    fn eq(&self, other: &Self) -> bool {
        // TODO: expand comparison to pointer.
        self.node == other.node && self.visited == other.visited
    }
}

#[derive(Debug)]
struct ParentVertex {
    id: usize,
    size: usize,
    max_size: usize,
    reverse: bool,
    prev: WeakParent,
    next: WeakParent,
    first: WeakVertex,
    last: WeakVertex,
}

impl ParentVertex {
    fn new(id: usize, max_size: usize) -> Self {
        Self {
            id,
            size: 0,
            max_size,
            reverse: false,
            prev: Weak::new(),
            next: Weak::new(),
            first: Weak::new(),
            last: Weak::new(),
        }
    }

    fn to_rc(self) -> RcParent {
        Rc::new(RefCell::new(self))
    }

    fn reverse(&mut self) {
        // TODO: two inner ifs have the same structure => potential refractor.
        if let (Some(first), Some(last)) = (self.first.upgrade(), self.last.upgrade()) {
            let (tmp_prev, tmp_next) = if self.reverse {
                (last.borrow().next.clone(), first.borrow().prev.clone())
            } else {
                (first.borrow().prev.clone(), last.borrow().next.clone())
            };

            if let (Some(prev_c), Some(prev_p)) = (tmp_prev.upgrade(), self.prev.upgrade()) {
                match (prev_p.borrow().reverse, self.reverse) {
                    (true, true) => {
                        // reverse, reverse => reverse, forward
                        prev_c.borrow_mut().prev = Rc::downgrade(&first);
                        first.borrow_mut().prev = Rc::downgrade(&prev_c);
                    }
                    (true, false) => {
                        // reverse, forward => reverse, reverse
                        prev_c.borrow_mut().prev = Rc::downgrade(&last);
                        last.borrow_mut().next = Rc::downgrade(&prev_c);
                    }
                    (false, true) => {
                        // forward, reverse => forward, forward
                        prev_c.borrow_mut().next = Rc::downgrade(&first);
                        first.borrow_mut().prev = Rc::downgrade(&prev_c);
                    }
                    (false, false) => {
                        // forward, forward => forward, reverse
                        prev_c.borrow_mut().next = Rc::downgrade(&last);
                        last.borrow_mut().next = Rc::downgrade(&prev_c);
                    }
                }
            }

            if let (Some(next_c), Some(next_p)) = (tmp_next.upgrade(), self.next.upgrade()) {
                match (self.reverse, next_p.borrow().reverse) {
                    (true, true) => {
                        // reverse, reverse => forward, reverse
                        last.borrow_mut().next = Rc::downgrade(&next_c);
                        next_c.borrow_mut().next = Rc::downgrade(&last);
                    }
                    (true, false) => {
                        // reverse, forward => forward, forward
                        last.borrow_mut().next = Rc::downgrade(&next_c);
                        next_c.borrow_mut().prev = Rc::downgrade(&last);
                    }
                    (false, true) => {
                        // forward, reverse => reverse, reverse
                        first.borrow_mut().prev = Rc::downgrade(&next_c);
                        next_c.borrow_mut().next = Rc::downgrade(&first);
                    }
                    (false, false) => {
                        // forward, forward => reverse, forward
                        first.borrow_mut().prev = Rc::downgrade(&next_c);
                        next_c.borrow_mut().prev = Rc::downgrade(&first);
                    }
                }
            }
        }

        self.reverse ^= true;
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
    use crate::tour::tests::test_tree_order;

    use super::super::tests::create_container;
    use super::*;

    #[test]
    fn test_init() {
        let n_nodes = 10;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);
        tree.init(None);

        assert_eq!(4, tree.parents.len());
        assert_eq!(n_nodes, tree.vertices.len());

        // First parent group
        let first_p = tree.parents[0].borrow();
        assert_eq!(3, first_p.size);
        assert!(first_p.first.upgrade().is_some());
        assert!(first_p.last.upgrade().is_some());

        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices[0]),
            &first_p.first
        ));
        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices[2]),
            &first_p.last
        ));

        // Last parent group
        let last_p = tree.parents.last().unwrap().borrow();
        assert_eq!(1, last_p.size);
        assert!(last_p.first.upgrade().is_some());
        assert!(last_p.last.upgrade().is_some());
        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices.last().unwrap()),
            &last_p.first
        ));
        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.vertices.last().unwrap()),
            &last_p.last
        ));

        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.parents.last().unwrap()),
            &tree.parents.first().unwrap().borrow().prev
        ));

        assert!(Weak::ptr_eq(
            &Rc::downgrade(&tree.parents.first().unwrap()),
            &tree.parents.last().unwrap().borrow().next
        ));
    }

    #[test]
    fn test_next_and_prev() {
        let n_nodes = 10;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);
        tree.init(None);

        let order = (0..n_nodes).collect();
        test_tree_order(&tree, &order);

        let order = vec![9, 1, 2, 4, 6, 3, 5, 8, 0, 7];
        tree.init(Some(&order));
        test_tree_order(&tree, &order);
    }

    #[test]
    fn test_between_forward() {
        let n_nodes = 10;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);
        tree.init(None);

        //  0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9

        // All vertices reside under the same parent node.
        assert!(tree.between(0, 1, 2)); // true
        assert!(!tree.between(0, 2, 1)); // false
        assert!(!tree.between(2, 1, 0)); // false
        assert!(tree.between(2, 0, 1)); // true

        // All vertices reside under distinct parent node.
        assert!(tree.between(2, 3, 7)); // true
        assert!(!tree.between(2, 7, 3)); // true
        assert!(!tree.between(7, 3, 2)); // false
        assert!(tree.between(7, 2, 3)); // true

        // Two out of three vertices reside under the same parent node.
        assert!(tree.between(3, 5, 8)); // true
        assert!(!tree.between(3, 8, 5)); // false
        assert!(!tree.between(8, 5, 3)); // false
        assert!(tree.between(8, 3, 5)); // true
    }

    #[test]
    fn test_parent_reverse() {
        let n_nodes = 10;
        let container = create_container(n_nodes);
        let mut tree = TwoLevelTree::new(&container, 3);

        tree.init(None);

        // 0 -> 1 -> 2 -> 5 -> 4 -> 3 -> 6 -> 7 -> 8 -> 9
        tree.parents[1].borrow_mut().reverse();
        let order = vec![0, 1, 2, 5, 4, 3, 6, 7, 8, 9];
        test_tree_order(&tree, &order);

        // 0 -> 1 -> 2 -> 5 -> 4 -> 3 -> 8 -> 7 -> 6 -> 9
        tree.parents[2].borrow_mut().reverse();
        let order = vec![0, 1, 2, 5, 4, 3, 8, 7, 6, 9];
        test_tree_order(&tree, &order);

        tree.parents[3].borrow_mut().reverse();
        test_tree_order(&tree, &order);

        // 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9
        tree.parents[1].borrow_mut().reverse();
        tree.parents[2].borrow_mut().reverse();
        let order = (0..10).collect();
        test_tree_order(&tree, &order);
    }
}
