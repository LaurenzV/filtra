use filtra_core::IntRect;

use crate::plan::{ImageId, Step};

/// Assigns the images of `steps`, which cover `rects`, to as few buffers as possible.
pub(crate) fn assign_buffers(steps: &[Step], rects: &[IntRect], output: ImageId) -> Buffers {
    // The reads of all steps, each step's ending at its entry in `ends`.
    let mut reads = Vec::new();
    let mut ends = Vec::with_capacity(steps.len());
    let mut last_reads: Vec<Option<usize>> = vec![None; rects.len()];
    let mut written = vec![false; rects.len()];

    for (index, step) in steps.iter().enumerate() {
        let start = reads.len();
        reads.extend(step.reads());
        ends.push(reads.len());

        for read in &reads[start..] {
            last_reads[read.0] = Some(index);
        }

        let dst = step.writes();
        assert!(!written[dst.0], "{dst:?} is written twice");
        written[dst.0] = true;
    }

    last_reads[output.0] = Some(usize::MAX);

    let mut pool = Pool::default();
    let mut assigned: Vec<Option<usize>> = vec![None; rects.len()];
    let mut start = 0;

    for (index, (step, &end)) in steps.iter().zip(&ends).enumerate() {
        let (dst, reads) = (step.writes(), &reads[start..end]);
        start = end;

        for read in reads {
            assert!(
                assigned[read.0].is_some() || !written[read.0],
                "{read:?} is read before it's written"
            );
        }

        assigned[dst.0] = Some(pool.allocate(rects[dst.0]));

        // A step may read an image more than once, so clearing its last read keeps its buffer from
        // being freed twice.
        for &read in reads {
            if last_reads[read.0] == Some(index) {
                pool.free.extend(assigned[read.0]);
                last_reads[read.0] = None;
            }
        }

        if last_reads[dst.0].is_none() {
            pool.free.extend(assigned[dst.0]);
        }
    }

    Buffers {
        count: pool.sizes.len(),
        images: assigned,
    }
}

/// The buffers that hold the images of a list of steps.
pub(crate) struct Buffers {
    /// The number of buffers.
    pub(crate) count: usize,
    /// The buffer of each image that a step writes, indexed by [`ImageId`].
    pub(crate) images: Vec<Option<usize>>,
}

#[derive(Default)]
struct Pool {
    /// The largest number of pixels each buffer holds.
    sizes: Vec<u64>,
    free: Vec<usize>,
}

impl Pool {
    /// A free buffer for `rect`, preferring the smallest one that is large enough.
    fn allocate(&mut self, rect: IntRect) -> usize {
        let size = u64::from(rect.width()) * u64::from(rect.height());
        let sizes = &self.sizes;
        let fitting = (self.free.iter().enumerate())
            .filter(|&(_, &buffer)| sizes[buffer] >= size)
            .min_by_key(|&(_, &buffer)| sizes[buffer])
            .map(|(index, _)| index);

        let buffer = match fitting.or((!self.free.is_empty()).then_some(0)) {
            Some(index) => self.free.swap_remove(index),
            None => {
                self.sizes.push(0);
                self.sizes.len() - 1
            }
        };

        self.sizes[buffer] = self.sizes[buffer].max(size);

        buffer
    }
}

#[cfg(all(test, feature = "all"))]
mod tests {
    use super::*;

    const RECT: IntRect = IntRect::new(0, 0, 10, 10);

    #[test]
    fn reuses_buffers_along_a_chain() {
        let steps: Vec<Step> = (1..=10).map(|index| offset(index - 1, index)).collect();

        let buffers = checked_assign(&steps, &[RECT; 11], ImageId(10));
        assert_eq!(buffers.count, 2);
        assert_eq!(buffers.images[0], None);
    }

    #[test]
    fn frees_images_read_twice_by_a_step_once() {
        // Images 2 and 3 are needed at the same time, so they can't share the buffer of image 0.
        let steps = [
            flood(0),
            merge(&[0, 0], 1),
            flood(2),
            flood(3),
            merge(&[1, 2, 3], 4),
        ];

        checked_assign(&steps, &[RECT; 5], ImageId(4));
    }

    #[test]
    #[should_panic(expected = "is read before it's written")]
    fn rejects_reading_images_written_later() {
        assign_buffers(&[offset(1, 0), flood(1)], &[RECT; 2], ImageId(0));
    }

    fn flood(dst: usize) -> Step {
        Step::Flood {
            dst: ImageId(dst),
            color: [0; 4],
        }
    }

    fn offset(src: usize, dst: usize) -> Step {
        Step::Offset {
            src: ImageId(src),
            dst: ImageId(dst),
            dx: 1,
            dy: 1,
        }
    }

    fn merge(srcs: &[usize], dst: usize) -> Step {
        Step::Merge {
            srcs: srcs.iter().copied().map(ImageId).collect(),
            dst: ImageId(dst),
        }
    }

    fn checked_assign(steps: &[Step], rects: &[IntRect], output: ImageId) -> Buffers {
        let buffers = assign_buffers(steps, rects, output);
        let mut held: Vec<Option<ImageId>> = vec![None; buffers.count];

        for step in steps {
            let reads: Vec<ImageId> = step.reads().collect();
            let dst = buffers.images[step.writes().0].unwrap();

            for &read in &reads {
                if let Some(buffer) = buffers.images[read.0] {
                    assert_eq!(held[buffer], Some(read), "{step:?}");
                    assert_ne!(buffer, dst, "{step:?}");
                }
            }

            held[dst] = Some(step.writes());
        }

        assert_eq!(held[buffers.images[output.0].unwrap()], Some(output));

        buffers
    }
}
