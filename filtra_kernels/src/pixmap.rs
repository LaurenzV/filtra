use crate::IntRect;

/// An image of premultiplied RGBA8 pixels, stored row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixmap {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Pixmap {
    /// Creates a transparent black pixmap.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; width as usize * height as usize * 4],
        }
    }

    /// Creates a pixmap from premultiplied RGBA8 data.
    ///
    /// Panics if its length doesn't match.
    pub fn from_data(data: Vec<u8>, width: u32, height: u32) -> Self {
        check_len(data.len(), width, height);

        Self {
            width,
            height,
            data,
        }
    }

    /// The width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The rectangle covering every pixel.
    pub fn bounds(&self) -> IntRect {
        bounds(self.width, self.height)
    }

    /// The premultiplied RGBA8 data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The premultiplied RGBA8 data, mutably.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Returns the premultiplied RGBA8 data.
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// The pixmap placed with its top-left pixel at `(x, y)`, as a kernel input.
    pub fn source(&self, x: i32, y: i32) -> Source<'_> {
        Source::new(&self.data, self.width, self.height, x, y)
    }

    /// The pixmap placed with its top-left pixel at `(x, y)`, as a kernel output.
    pub fn target(&mut self, x: i32, y: i32) -> Target<'_> {
        Target::new(&mut self.data, self.width, self.height, x, y)
    }

    /// The pixmap as a [`PixmapMut`].
    pub fn as_mut(&mut self) -> PixmapMut<'_> {
        PixmapMut {
            width: self.width,
            height: self.height,
            data: &mut self.data,
        }
    }
}

/// Premultiplied RGBA8 pixels borrowed from elsewhere, stored row by row.
#[derive(Debug)]
pub struct PixmapMut<'a> {
    width: u32,
    height: u32,
    data: &'a mut [u8],
}

impl<'a> PixmapMut<'a> {
    /// Borrows premultiplied RGBA8 data as a pixmap.
    ///
    /// Panics if its length doesn't match.
    pub fn from_data(data: &'a mut [u8], width: u32, height: u32) -> Self {
        check_len(data.len(), width, height);

        Self {
            width,
            height,
            data,
        }
    }

    /// The width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The rectangle covering every pixel.
    pub fn bounds(&self) -> IntRect {
        bounds(self.width, self.height)
    }

    /// The pixmap placed with its top-left pixel at `(x, y)`, as a kernel input.
    pub fn source(&self, x: i32, y: i32) -> Source<'_> {
        Source::new(self.data, self.width, self.height, x, y)
    }

    /// The pixmap placed with its top-left pixel at `(x, y)`, as a kernel output.
    pub fn target(&mut self, x: i32, y: i32) -> Target<'_> {
        Target::new(self.data, self.width, self.height, x, y)
    }
}

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

fn bounds(width: u32, height: u32) -> IntRect {
    IntRect::new(
        0,
        0,
        width.min(i32::MAX as u32) as i32,
        height.min(i32::MAX as u32) as i32,
    )
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
    fn new(data: &'a mut [u8], width: u32, height: u32, x: i32, y: i32) -> Self {
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
pub(crate) fn segments(
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
