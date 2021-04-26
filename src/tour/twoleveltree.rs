use std::{cell::RefCell, rc::Rc};
use std::fmt;

use crate::node::{Container, Node};

use super::{Tour, Vertex};

type RcChild = Rc<RefCell<InnerChild>>;
type RcParent = Rc<RefCell<InnerParent>>;

#[derive(Debug)]
pub struct TwoLevelTree<'a> {
    container: &'a Container,
    groupsize: usize,
    parents: Vec<ParentVertex>,
    vertices: Vec<TlVertex>,
}

impl<'a> TwoLevelTree<'a> {
    pub fn new(container: &'a Container, groupsize: usize) -> Self {
        let n_segments = (container.len() / groupsize) + 1;
        let mut parents = Vec::with_capacity(n_segments);
        
        for ii in 0..n_segments {
            let p = ParentVertex::new(ii);
            
        }
        
        Self {
            container,
            groupsize,
            parents,
            vertices: Vec::new(),
        }
    }

}

impl<'a> Tour for TwoLevelTree<'a> {
    type Output = TlVertex;

    fn get(&self, node_idx: usize) -> Option<&Self::Output> {
        self.vertices.get(node_idx)
    }

    fn next(&self, node_idx: usize) -> Option<&Self::Output> {
        if node_idx > self.vertices.len() {
            return None;
        }

        if let Some(idx) = self.vertices[node_idx].next_id() {
            self.vertices.get(idx)
        } else {
            None
        }
    }

    fn prev(&self, node_idx: usize) -> Option<&Self::Output> {
        if node_idx > self.vertices.len() {
            return None;
        }

        if let Some(idx) = self.vertices[node_idx].prev_id() {
            self.vertices.get(idx)
        } else {
            None
        }
    }

    fn between(&self, _from_idx: usize, _mid_idx: usize, _to_idx: usize) -> bool {
        todo!()
    }

    fn flip(&mut self, _from_idx1: usize, _to_idx1: usize, _from_idx2: usize, _to_idx2: usize) {
        todo!()
    }
}

#[derive(Clone, Debug)]
struct ParentVertex {
    inner: RcParent,
}

impl ParentVertex {
    fn new(id: usize) -> Self {
        let inner = Rc::new(RefCell::new(InnerParent {
            id,
            len: 0,
            reverse: false,
            first: None,
            last: None,
            prev: None,
            next: None,
        }));

        Self {
            inner,
        }
    }

    fn id(&self) -> usize {
        self.inner.borrow().id
    }

    fn len(&self) -> usize {
        self.inner.borrow().len
    }

    /// Returns the first element of this segment, taking into account the reversal bit flag.
    fn first(&self) -> Option<RcChild> {
        if self.inner.borrow().reverse {
            self.inner.borrow().last.clone()
        } else {
            self.inner.borrow().first.clone()
        }
    }

    /// Returns the last element of this segment, taking into account the reversal bit flag
    fn last(&self) -> Option<RcChild> {
        if self.inner.borrow().reverse {
            self.inner.borrow().first.clone()
        } else {
            self.inner.borrow().last.clone()
        }
    }

    fn prev(&self) -> Option<RcParent> {
        self.inner.borrow().prev.clone()
    }

    fn insert_prev(&self, new_prev: &RcParent) {
        // TODO: check whether new_prev is an isolated node.
        if let Some(old_prev) = self.prev() {
            old_prev.borrow_mut().next = Some(new_prev.clone());
            new_prev.borrow_mut().prev = Some(old_prev.clone());
        }

        new_prev.borrow_mut().next = Some(self.inner.clone());
        self.inner.borrow_mut().prev = Some(new_prev.clone());
    }

    fn next(&self) -> Option<RcParent> {
        self.inner.borrow().next.clone()
    }

    fn insert_next(&self, new_next: &RcParent) {
        // TODO: check whether new_prev is an isolated node.
        if let Some(old_next) = self.next() {
            old_next.borrow_mut().prev = Some(new_next.clone());
            new_next.borrow_mut().next = Some(old_next.clone());
        }

        new_next.borrow_mut().prev = Some(self.inner.clone());
        self.inner.borrow_mut().next = Some(new_next.clone());
    }

    fn is_reverse(&self) -> bool {
        self.inner.borrow().reverse
    }
}

struct InnerParent {
    /// The id of the parent node in the two-level tree.
    id: usize,

    /// Indicates how many children nodes the parent node has.
    len: usize,

    /// The flag indicates in which direction the traversal of vertices contained in this parent
    /// node should take place.
    ///
    /// If the flag is set to `false`, the direction is forward. Otherwise, the vertices are
    /// traversed in the reverse order. The default value is `false`.
    reverse: bool,

    /// The first child node of the segment, in the forward direction.
    first: Option<RcChild>,

    /// The last child node of the segment, in the forward direction.
    last: Option<RcChild>,    

    /// The reference to the the parent node preceding this segment in the tree.
    prev: Option<RcParent>,

    /// The reference to the parent node succeeding this segment in the tree.
    next: Option<RcParent>,
}

impl fmt::Debug for InnerParent {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Representation of vertex in the two-level-tree data structure.
#[derive(Debug, PartialEq)]
pub struct TlVertex {
    inner: RcChild,    
}

impl TlVertex {
    fn new(node: &Node, parent: &ParentVertex) -> Self {
        let inner = Rc::new(RefCell::new(InnerChild {
            node: node.clone(),
            next_id: None,
            prev_id: None,
            parent: parent.clone(),
        }));

        Self {
            inner,
        }
    }

    /// Returns the `id` of the next vertex in the tour, taking into account the reversal bit flag
    /// of the parent node.
    fn next_id(&self) -> Option<usize> {
        if self.inner.borrow().parent.is_reverse() {
            self.inner.borrow().prev_id
        } else {
            self.inner.borrow().next_id
        }
    }

    fn prev_id(&self) -> Option<usize> {
        if self.inner.borrow().parent.is_reverse() {
            self.inner.borrow().prev_id
        } else {
            self.inner.borrow().next_id
        }
    }
}

impl Vertex for TlVertex {}

struct InnerChild {
    node: Node,
    next_id: Option<usize>,
    prev_id: Option<usize>,
    parent: ParentVertex,
}

impl fmt::Debug for InnerChild {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl PartialEq for InnerChild {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}