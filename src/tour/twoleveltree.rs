use std::{cell::RefCell, rc::Rc};

use crate::node::{Container, Node};

use super::{Tour, Vertex};

type RcVertex = Rc<RefCell<TltVertex>>;
type RcParent = Rc<RefCell<ParentVertex>>;

#[derive(Debug)]
pub struct TwoLevelTree<'a> {
    max_groupsize: usize,
    container: &'a Container,
    vertices: Vec<RcVertex>,
    parents: Vec<RcParent>,
}

impl<'a> TwoLevelTree<'a> {
    pub fn new(container: &'a Container, max_groupsize: usize) -> Self {
        let mut result = Self {
            max_groupsize,
            container,
            vertices: Vec::new(),
            parents: Vec::new(),
        };

        result.init();
        result
    }

    fn init(&mut self) {
        

        let n_parents = (self.container.len() / self.max_groupsize) + 1;
        let mut parents = Vec::with_capacity(n_parents);
        let p = create_parent(0, self.max_groupsize);
        parents.push(p);

        for ii in 1..n_parents {
            let p = create_parent(ii, self.max_groupsize);
            let last = parents.last().unwrap();
            link_parents(last, &p);
            parents.push(p);
        }

        self.parents = parents;
    }

}

impl<'a> Tour for TwoLevelTree<'a> {
    type Output = TltVertex;

    fn get(&self, node_idx: usize) -> Option<&Self::Output> {
        if let Some(v) = self.vertices.get(node_idx) {
            unsafe { return v.as_ref().as_ptr().as_ref(); }
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
    next: Option<RcVertex>,
    prev: Option<RcVertex>,
    parent: Option<RcParent>,
}

impl TltVertex {
    pub fn new(node: &Node) -> Self {
        Self {
            node: node.clone(),
            next: None,
            prev: None,
            parent: None
        }
    }
}

impl Vertex for TltVertex {}

impl PartialEq for TltVertex {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

#[derive(Debug)]
struct ParentVertex {
    id: usize,
    size: usize,
    max_size: usize,
    reverse: bool,
    first: Option<RcVertex>,
    last: Option<RcVertex>,
    next: Option<RcParent>,
    prev: Option<RcParent>,
}

impl ParentVertex {
    pub fn new(id: usize, max_size: usize) -> Self {
        Self {
            id,
            size: 0,
            max_size,
            reverse: false,
            first: None,
            last: None,
            next: None,
            prev: None,
        }
    }

    pub fn prev(&self) -> Option<&RcParent> {
        if self.reverse {
            self.next.as_ref()
        } else {
            self.prev.as_ref()
        }
    }

    pub fn next(&self) -> Option<&RcParent> {
        if self.reverse {
            self.prev.as_ref()
        } else {
            self.next.as_ref()
        }
    }

    pub fn first(&self) -> Option<&RcVertex> {
        if self.reverse {
            self.last.as_ref()
        } else {
            self.first.as_ref()
        }
    }

    pub fn last(&self) -> Option<&RcVertex> {
        if self.reverse {
            self.first.as_ref()
        } else {
            self.last.as_ref()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

fn push_child(parent: &RcParent, child: &RcVertex) {
    child.borrow_mut().parent = Some(parent.clone());

    if parent.borrow().is_empty() {
        parent.borrow_mut().first = Some(child.clone());
        parent.borrow_mut().last = Some(child.clone());
    } else {
        if parent.borrow().reverse {
            parent.borrow().first().unwrap().borrow_mut().prev = Some(child.clone());
            child.borrow_mut().next = parent.borrow().first.clone();
        } else {
            parent.borrow().last().unwrap().borrow_mut().next = Some(child.clone());
            child.borrow_mut().prev = parent.borrow().last.clone();
        }
    }

    if let Some(next) = parent.borrow().next() {
        link_children_x(Some(&child), parent.borrow().reverse, next.borrow().first(), next.borrow().reverse);
    }

    parent.borrow_mut().size += 1;
}

fn link_parents(prev: &RcParent, next: &RcParent) {
    let (revp, revn) = (prev.borrow().reverse, next.borrow().reverse);
    match (revp, revn) {
        (true, true) => {
            // reversed, reversed
            prev.borrow_mut().prev = Some(next.clone());
            next.borrow_mut().next = Some(prev.clone());
        }
        (true, false) => {
            // reversed, forward
            prev.borrow_mut().prev = Some(next.clone());
            next.borrow_mut().prev = Some(prev.clone());
        }
        (false, true) => {
            // forward, reversed
            prev.borrow_mut().next = Some(next.clone());
            next.borrow_mut().next = Some(prev.clone());
        }
        (false, false) => {
            // forward, forward
            prev.borrow_mut().next = Some(next.clone());
            next.borrow_mut().prev = Some(prev.clone());
        }
    }

    link_children_x(prev.borrow().last(), revp, next.borrow().first(), revn)
}

/// Link children nodes across segments.
fn link_children_x(plc: Option<&RcVertex>, revp: bool,nfc: Option<&RcVertex>, revn: bool) {
    // plc = previous node's last child
    // nfc = next node's first child
    if let (Some(p), Some(n)) = (plc, nfc) {
        match (revp, revn) {
            (true, true) => {
                // reversed, reversed
                p.borrow_mut().prev = Some(n.clone());
                n.borrow_mut().next = Some(p.clone());
            }
            (true, false) => {
                // reversed, forward
                p.borrow_mut().prev = Some(n.clone());
                n.borrow_mut().prev = Some(p.clone());
            }
            (false, true) => {
                // forward, reversed
                p.borrow_mut().next = Some(n.clone());
                n.borrow_mut().prev = Some(p.clone());
            }
            (false, false) => {
                // forward, forward
                p.borrow_mut().next = Some(n.clone());
                n.borrow_mut().prev = Some(p.clone());
            }
        }
    }
}

fn create_parent(id: usize, max_size: usize) -> RcParent {
    Rc::new(RefCell::new(
        ParentVertex::new(id, max_size)
    ))
}

fn create_vertex(node: &Node) -> RcVertex {
    Rc::new(RefCell::new(
        TltVertex::new(node)
    ))
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::*;
    use super::super::tests::create_container;

    #[test]
    fn test_push_child() {
        let n_nodes = 5;
        let container = create_container(n_nodes);
        let vertices: Vec<RcVertex> = container.into_iter().map(|node| create_vertex(&node)).collect();

        let p = create_parent(0, 3);
        vertices.iter().for_each(|v| push_child(&p, v));
        
        assert_eq!(n_nodes, p.borrow().size);
    }

    #[test]
    fn test_link_parents() {
        let p1 = create_parent(0, 3);
        let p2 = create_parent(1, 3);

        // p1 -> p2
        link_parents(&p1, &p2);

        assert!(p1.borrow().prev().is_none());
        assert!(p1.borrow().next().is_some());
        assert!(p2.borrow().prev().is_some());
        assert!(p2.borrow().next().is_none());
        assert_eq!(p1.borrow().id, p2.borrow().prev().unwrap().borrow().id);
        assert_eq!(p2.borrow().id, p1.borrow().next().unwrap().borrow().id);
    }
}