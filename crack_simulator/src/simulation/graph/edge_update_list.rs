use std::collections::VecDeque;
use super::{EdgeIndex, EdgeMatrix};
use super::Edge;

pub struct EdgeUpdateList {
    v: VecDeque<EdgeIndex>
}

impl EdgeUpdateList {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            v: VecDeque::with_capacity(capacity)
        }
    }

    pub(super) fn pop(&mut self) -> Option<EdgeIndex> {
        self.v.pop_front()
    }

    pub(super) fn push(&mut self, index: EdgeIndex) {
        self.v.push_back(index);
    }
}