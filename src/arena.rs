use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { items: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Arena {
            items: Vec::with_capacity(cap),
        }
    }

    pub fn alloc(&mut self, val: T) -> NodeId {
        let id = self.items.len();
        self.items.push(val);
        NodeId(id)
    }

    pub fn get(&self, id: NodeId) -> &T {
        &self.items[id.0]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut T {
        &mut self.items[id.0]
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.items.iter().enumerate().map(|(i, v)| (NodeId(i), v))
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena::new()
    }
}

impl<T> Index<NodeId> for Arena<T> {
    type Output = T;
    fn index(&self, id: NodeId) -> &T {
        self.get(id)
    }
}

impl<T> IndexMut<NodeId> for Arena<T> {
    fn index_mut(&mut self, id: NodeId) -> &mut T {
        self.get_mut(id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    pub const INVALID: NodeId = NodeId(usize::MAX);
}
