//! The CPU backend.

mod executor;
mod pixmap;

use std::cmp::Reverse;
use std::fmt;

pub use pixmap::PixmapMut;

/// Reusable memory for the intermediate images of [`apply`](crate::apply).
pub struct BufferPool {
    buffers: Vec<Vec<u8>>,
    max_buffers: usize,
}

impl BufferPool {
    /// Creates an empty pool that keeps up to `max_buffers` buffers between calls.
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

    fn take(&mut self, count: usize) -> Vec<Vec<u8>> {
        let rest = self.buffers.split_off(count.min(self.buffers.len()));
        let mut buffers = std::mem::replace(&mut self.buffers, rest);
        buffers.resize_with(count, Vec::new);

        buffers
    }

    fn keep(&mut self, buffers: Vec<Vec<u8>>) {
        self.buffers.extend(buffers);
        self.buffers.retain(|buffer| buffer.capacity() > 0);
        self.buffers
            .sort_unstable_by_key(|buffer| Reverse(buffer.capacity()));
        self.buffers.truncate(self.max_buffers);
    }
}

impl fmt::Debug for BufferPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufferPool")
            .field("buffers", &self.buffers.len())
            .field("memory", &self.memory())
            .field("max_buffers", &self.max_buffers)
            .finish()
    }
}
