use crate::{ImageId, IntRect, Pass};

/// Assigns the images of `passes`, which cover `rects`, to as few buffers as possible.
///
/// Each image must be written by exactly one pass before any pass reads it.
pub fn assign_buffers<P: Pass>(passes: &[P], rects: &[IntRect], output: ImageId) -> Buffers {
    // The reads of all passes, each pass's ending at its entry in `ends`.
    let mut reads = Vec::new();
    let mut ends = Vec::with_capacity(passes.len());
    let mut last_reads: Vec<Option<usize>> = vec![None; rects.len()];

    for (index, pass) in passes.iter().enumerate() {
        let start = reads.len();
        reads.extend(pass.reads());
        ends.push(reads.len());

        for read in &reads[start..] {
            last_reads[read.0] = Some(index);
        }
    }

    last_reads[output.0] = Some(usize::MAX);

    let mut pool = Pool::default();
    let mut assigned: Vec<Option<usize>> = vec![None; rects.len()];
    let mut start = 0;

    for (index, (pass, &end)) in passes.iter().zip(&ends).enumerate() {
        let (dst, reads) = (pass.writes(), &reads[start..end]);
        start = end;

        for read in reads {
            assert!(
                assigned[read.0].is_some(),
                "{read:?} is read before it's written"
            );
        }

        assert!(assigned[dst.0].is_none(), "{dst:?} is written twice");

        assigned[dst.0] = Some(pool.allocate(rects[dst.0]));

        // A pass may read an image more than once, so clearing its last read keeps its buffer from
        // being freed twice.
        for &read in reads {
            if last_reads[read.0] == Some(index) {
                pool.free.push(assigned[read.0].unwrap());
                last_reads[read.0] = None;
            }
        }

        if last_reads[dst.0].is_none() {
            pool.free.push(assigned[dst.0].unwrap());
        }
    }

    Buffers {
        count: pool.sizes.len(),
        images: (assigned.into_iter())
            .map(|buffer| buffer.expect("every image is written"))
            .collect(),
    }
}

/// The buffers that hold the images of a list of passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffers {
    /// The number of buffers.
    pub count: usize,
    /// The buffer of each image, indexed by [`ImageId`].
    pub images: Vec<usize>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Step;

    const RECT: IntRect = IntRect::new(0, 0, 10, 10);

    #[test]
    fn reuses_buffers_along_a_chain() {
        let mut passes = vec![Step::SourceGraphic { dst: ImageId(0) }];

        for index in 1..=10 {
            passes.push(Step::Offset {
                src: ImageId(index - 1),
                dst: ImageId(index),
                dx: 1,
                dy: 1,
            });
        }

        let buffers = checked_assign(&passes, &[RECT; 11], ImageId(10));
        assert_eq!(buffers.count, 2);
    }

    #[test]
    fn frees_images_read_twice_by_a_pass_once() {
        let flood = |dst| Step::Flood {
            dst: ImageId(dst),
            color: [0; 4],
        };

        // Images 2 and 3 are needed at the same time, so they can't share the buffer of image 0.
        let passes = [
            Step::SourceGraphic { dst: ImageId(0) },
            merge(&[0, 0], 1),
            flood(2),
            flood(3),
            merge(&[1, 2, 3], 4),
        ];

        checked_assign(&passes, &[RECT; 5], ImageId(4));
    }

    #[test]
    #[should_panic(expected = "is read before it's written")]
    fn rejects_reading_unwritten_images() {
        let passes = [Step::Offset {
            src: ImageId(1),
            dst: ImageId(0),
            dx: 0,
            dy: 0,
        }];

        assign_buffers(&passes, &[RECT; 2], ImageId(0));
    }

    fn merge(srcs: &[usize], dst: usize) -> Step {
        Step::Merge {
            srcs: srcs.iter().copied().map(ImageId).collect(),
            dst: ImageId(dst),
        }
    }

    fn checked_assign(passes: &[Step], rects: &[IntRect], output: ImageId) -> Buffers {
        let buffers = assign_buffers(passes, rects, output);
        let mut held: Vec<Option<ImageId>> = vec![None; buffers.count];

        for pass in passes {
            let reads: Vec<ImageId> = pass.reads().collect();
            let dst = buffers.images[pass.writes().0];

            for &read in &reads {
                assert_eq!(held[buffers.images[read.0]], Some(read), "{pass:?}");
            }

            assert!(
                reads.iter().all(|&read| buffers.images[read.0] != dst),
                "{pass:?}"
            );

            held[dst] = Some(pass.writes());
        }

        assert_eq!(held[buffers.images[output.0]], Some(output));

        buffers
    }
}
