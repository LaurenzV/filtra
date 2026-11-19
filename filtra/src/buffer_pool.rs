use std::cmp::Reverse;

/// Reusable memory for the intermediate images of [`apply`](crate::apply).
#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<Vec<u8>>,
    max_buffers: usize,
}

impl BufferPool {
    /// Creates an empty pool that keeps up to `max_buffers` buffers.
    pub fn new(max_buffers: usize) -> Self {
        Self {
            buffers: Vec::new(),
            max_buffers,
        }
    }

    /// The number of bytes held.
    pub fn memory(&self) -> usize {
        self.buffers.iter().map(Vec::capacity).sum()
    }

    /// Frees buffers, largest first, until at most `bytes` are held.
    pub fn shrink_to(&mut self, bytes: usize) {
        while self.memory() > bytes {
            self.buffers.remove(0);
        }
    }

    pub(crate) fn take(&mut self, count: usize) -> Vec<Vec<u8>> {
        let rest = self.buffers.split_off(count.min(self.buffers.len()));
        let mut buffers = std::mem::replace(&mut self.buffers, rest);
        buffers.resize_with(count, Vec::new);

        buffers
    }

    pub(crate) fn keep(&mut self, buffers: Vec<Vec<u8>>) {
        self.buffers.extend(buffers);
        self.keep_largest();
    }

    fn keep_largest(&mut self) {
        self.buffers.retain(|buffer| buffer.capacity() > 0);
        self.buffers
            .sort_unstable_by_key(|buffer| Reverse(buffer.capacity()));
        self.buffers.truncate(self.max_buffers);
    }
}
