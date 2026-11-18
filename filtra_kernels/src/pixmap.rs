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
    width: u32,
    rect: IntRect,
}

impl<'a> Source<'a> {
    fn new(data: &'a [u8], width: u32, height: u32, x: i32, y: i32) -> Self {
        Self {
            data,
            width,
            rect: extent(x, y, width, height),
        }
    }

    /// The pixels the pixmap covers.
    pub fn rect(&self) -> IntRect {
        self.rect
    }

    pub(crate) fn row(&self, y: i32, x0: i32, x1: i32) -> Option<&'a [u8]> {
        let covered = (self.rect.y0..self.rect.y1).contains(&y)
            && x0 >= self.rect.x0
            && x1 <= self.rect.x1
            && x0 < x1;

        if !covered {
            return None;
        }

        let start = (y - self.rect.y0) as usize * stride(self.width);
        let columns = (x0 - self.rect.x0) as usize * 4..(x1 - self.rect.x0) as usize * 4;

        Some(&self.data[start..][columns])
    }

    pub(crate) fn pixel(&self, x: i32, y: i32) -> Option<&'a [u8; 4]> {
        let x = usize::try_from(i64::from(x) - i64::from(self.rect.x0)).ok()?;
        let y = usize::try_from(i64::from(y) - i64::from(self.rect.y0)).ok()?;
        let width = self.width as usize;

        if x >= width {
            return None;
        }

        let index = y.checked_mul(width)?.checked_add(x)?;

        self.data.as_chunks().0.get(index)
    }
}

/// A placed pixmap that a kernel writes completely.
#[derive(Debug)]
pub struct Target<'a> {
    data: &'a mut [u8],
    width: u32,
    rect: IntRect,
}

impl<'a> Target<'a> {
    fn new(data: &'a mut [u8], width: u32, height: u32, x: i32, y: i32) -> Self {
        Self {
            data,
            width,
            rect: extent(x, y, width, height),
        }
    }

    /// The pixels the pixmap covers.
    pub fn rect(&self) -> IntRect {
        self.rect
    }

    pub(crate) fn rows_mut(&mut self) -> impl Iterator<Item = (i32, &mut [u8])> {
        let rows = if self.rect.is_empty() {
            0
        } else {
            self.rect.height() as usize
        };

        let stride = stride(self.width);
        let width = self.rect.width() as usize * 4;

        self.data
            .chunks_exact_mut(stride)
            .take(rows)
            .zip(self.rect.y0..)
            .map(move |(row, y)| (y, &mut row[..width]))
    }

    pub(crate) fn repeat_rows(&mut self, period: usize) {
        let rows = if self.rect.is_empty() {
            0
        } else {
            self.rect.height() as usize
        };

        let stride = stride(self.width);
        let width = self.rect.width() as usize * 4;

        for row in period..rows {
            let start = (row - period) * stride;
            self.data.copy_within(start..start + width, row * stride);
        }
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
