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
        let len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4));

        assert_eq!(
            Some(data.len()),
            len,
            "the data length doesn't match the size"
        );

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
        IntRect::new(
            0,
            0,
            self.width.min(i32::MAX as u32) as i32,
            self.height.min(i32::MAX as u32) as i32,
        )
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
        Source {
            rect: extent(x, y, self.width, self.height),
            pixmap: self,
        }
    }

    /// The pixmap placed with its top-left pixel at `(x, y)`, as a kernel output.
    pub fn target(&mut self, x: i32, y: i32) -> Target<'_> {
        Target {
            rect: extent(x, y, self.width, self.height),
            pixmap: self,
        }
    }

    fn stride(&self) -> usize {
        (self.width as usize * 4).max(1)
    }
}

/// A placed pixmap that a kernel reads. Pixels outside of it are transparent black.
#[derive(Debug, Clone, Copy)]
pub struct Source<'a> {
    pixmap: &'a Pixmap,
    rect: IntRect,
}

impl<'a> Source<'a> {
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

        let start = (y - self.rect.y0) as usize * self.pixmap.stride();
        let columns = (x0 - self.rect.x0) as usize * 4..(x1 - self.rect.x0) as usize * 4;

        Some(&self.pixmap.data[start..][columns])
    }
}

/// A placed pixmap that a kernel writes completely.
#[derive(Debug)]
pub struct Target<'a> {
    pixmap: &'a mut Pixmap,
    rect: IntRect,
}

impl Target<'_> {
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

        let stride = self.pixmap.stride();
        let width = self.rect.width() as usize * 4;

        self.pixmap
            .data
            .chunks_exact_mut(stride)
            .take(rows)
            .zip(self.rect.y0..)
            .map(move |(row, y)| (y, &mut row[..width]))
    }
}

fn extent(x: i32, y: i32, width: u32, height: u32) -> IntRect {
    let end =
        |start: i32, len: u32| (i64::from(start) + i64::from(len)).min(i64::from(i32::MAX)) as i32;

    IntRect::new(x, y, end(x, width), end(y, height))
}
