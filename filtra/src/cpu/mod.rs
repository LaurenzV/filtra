//! The CPU backend.

mod executor;

use std::cmp::Reverse;

pub use filtra_kernels::{Pixmap, PixmapMut};

use crate::IntRect;
use filtra_plan::Plan;

/// A reusable backend for applying filters on the CPU.
#[derive(Debug)]
pub struct Backend {
    buffers: Vec<Vec<u8>>,
    max_buffers: usize,
}

impl Backend {
    /// Creates a backend that keeps up to `max_buffers` buffers between calls.
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

impl crate::Backend for Backend {
    type Target<'a> = PixmapMut<'a>;
    type Image<'a> = &'a mut [u8];

    fn bounds(target: &PixmapMut<'_>) -> IntRect {
        let bounds = target.bounds();
        IntRect::new(bounds.x0, bounds.y0, bounds.x1, bounds.y1)
    }

    fn execute(
        &mut self,
        plan: &Plan,
        target: PixmapMut<'_>,
        mut render_image: impl FnMut(usize, IntRect, &mut [u8]),
    ) {
        executor::execute(plan, target, self, &mut render_image);
    }
}
