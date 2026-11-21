use crate::IntRect;

fn check_len(len: usize, width: u32, height: u32) {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4));

    assert_eq!(
        Some(len),
        expected,
        "the data length doesn't match the size"
    );
}

fn stride(width: u32) -> usize {
    (width as usize * 4).max(1)
}

/// A placed pixmap that a kernel reads. Pixels outside of it are transparent black.
#[derive(Debug, Clone, Copy)]
pub struct Source<'a> {
    data: &'a [u8],
    layout: Layout,
}

impl<'a> Source<'a> {
    /// Places premultiplied RGBA8 data with its top-left pixel at `(x, y)`.
    ///
    /// Panics if its length doesn't match.
    pub fn new(data: &'a [u8], width: u32, height: u32, x: i32, y: i32) -> Self {
        check_len(data.len(), width, height);

        Self {
            data,
            layout: Layout::new(width, height, x, y),
        }
    }

    /// The pixels the pixmap covers.
    pub fn rect(&self) -> IntRect {
        self.layout.rect
    }

    /// The part of the pixmap within `rect`.
    pub fn clip(self, rect: IntRect) -> Self {
        Self {
            layout: self.layout.clip(rect),
            ..self
        }
    }

    pub(crate) fn row(&self, y: i32, x0: i32, x1: i32) -> Option<&'a [u8]> {
        let rect = self.layout.rect;
        let covered = (rect.y0..rect.y1).contains(&y) && x0 >= rect.x0 && x1 <= rect.x1 && x0 < x1;

        if !covered {
            return None;
        }

        let start = self.layout.index(x0, y);

        Some(&self.data[start..start + (x1 - x0) as usize * 4])
    }

    pub(crate) fn copy_row(&self, y: i64, x0: i64, out: &mut [u8]) {
        let rect = self.layout.rect;
        let x1 = x0.saturating_add((out.len() / 4) as i64);
        let from = x0.max(rect.x0.into()).min(x1);
        let to = x1.min(rect.x1.into()).max(from);
        let (start, stop) = ((from - x0) as usize * 4, (to - x0) as usize * 4);
        let row = i32::try_from(y)
            .ok()
            .and_then(|y| self.row(y, from as i32, to as i32));

        out[..start].fill(0);

        match row {
            Some(row) => out[start..stop].copy_from_slice(row),
            None => out[start..stop].fill(0),
        }

        out[stop..].fill(0);
    }

    pub(crate) fn pixel(&self, x: i32, y: i32) -> Option<&'a [u8; 4]> {
        let rect = self.layout.rect;
        let covered = (rect.x0..rect.x1).contains(&x) && (rect.y0..rect.y1).contains(&y);

        if !covered {
            return None;
        }

        self.data[self.layout.index(x, y)..].first_chunk()
    }
}

/// A placed pixmap that a kernel writes completely.
#[derive(Debug)]
pub struct Target<'a> {
    data: &'a mut [u8],
    layout: Layout,
}

impl<'a> Target<'a> {
    /// Places premultiplied RGBA8 data with its top-left pixel at `(x, y)`.
    ///
    /// Panics if its length doesn't match.
    pub fn new(data: &'a mut [u8], width: u32, height: u32, x: i32, y: i32) -> Self {
        check_len(data.len(), width, height);

        Self {
            data,
            layout: Layout::new(width, height, x, y),
        }
    }

    /// The pixels the pixmap covers.
    pub fn rect(&self) -> IntRect {
        self.layout.rect
    }

    /// The part of the pixmap within `rect`.
    pub fn clip(self, rect: IntRect) -> Self {
        Self {
            layout: self.layout.clip(rect),
            ..self
        }
    }

    pub(crate) fn rows_mut(&mut self) -> impl Iterator<Item = (i32, &mut [u8])> {
        let (rows, start) = self.layout.rows();
        let width = self.layout.rect.width() as usize * 4;

        self.data[start..]
            .chunks_mut(self.layout.stride)
            .take(rows)
            .zip(self.layout.rect.y0..)
            .map(move |(row, y)| (y, &mut row[..width]))
    }

    pub(crate) fn repeat_rows(&mut self, period: usize) {
        let (rows, start) = self.layout.rows();
        let (width, stride) = (self.layout.rect.width() as usize * 4, self.layout.stride);

        for row in period..rows {
            let from = start + (row - period) * stride;
            self.data
                .copy_within(from..from + width, start + row * stride);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Layout {
    stride: usize,
    origin: (i32, i32),
    rect: IntRect,
}

impl Layout {
    fn new(width: u32, height: u32, x: i32, y: i32) -> Self {
        Self {
            stride: stride(width),
            origin: (x, y),
            rect: extent(x, y, width, height),
        }
    }

    fn clip(self, rect: IntRect) -> Self {
        Self {
            rect: self.rect.intersect(&rect),
            ..self
        }
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (y - self.origin.1) as usize * self.stride + (x - self.origin.0) as usize * 4
    }

    fn rows(&self) -> (usize, usize) {
        if self.rect.is_empty() {
            return (0, 0);
        }

        (
            self.rect.height() as usize,
            self.index(self.rect.x0, self.rect.y0),
        )
    }
}

#[inline(always)]
pub(crate) fn for_each_segment(
    dst: &mut Target<'_>,
    rects: impl Iterator<Item = IntRect> + Clone,
    mut f: impl FnMut(i32, i32, i32, &mut [u8]),
) {
    let rect = dst.rect();

    for (y, row) in dst.rows_mut() {
        for (x0, x1) in segments(rect.x0, rect.x1, rects.clone()) {
            f(
                y,
                x0,
                x1,
                &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4],
            );
        }
    }
}

#[inline(always)]
fn segments(
    x0: i32,
    x1: i32,
    rects: impl Iterator<Item = IntRect> + Clone,
) -> impl Iterator<Item = (i32, i32)> {
    let mut x = x0;

    // Plain loops, so that this inlines fully into the SIMD code that iterates the segments.
    core::iter::from_fn(
        #[inline(always)]
        move || {
            if x >= x1 {
                return None;
            }

            let mut end = x1;

            for rect in rects.clone() {
                if rect.is_empty() {
                    continue;
                }

                for edge in [rect.x0, rect.x1] {
                    if edge > x && edge < end {
                        end = edge;
                    }
                }
            }

            let segment = (x, end);
            x = end;

            Some(segment)
        },
    )
}

fn extent(x: i32, y: i32, width: u32, height: u32) -> IntRect {
    let end =
        |start: i32, len: u32| (i64::from(start) + i64::from(len)).min(i64::from(i32::MAX)) as i32;

    IntRect::new(x, y, end(x, width), end(y, height))
}
