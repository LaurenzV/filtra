use filtra_kernels::{Source, Target};

use crate::IntRect;

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

    pub(crate) fn bounds(&self) -> IntRect {
        IntRect::new(
            0,
            0,
            self.width.min(i32::MAX as u32) as i32,
            self.height.min(i32::MAX as u32) as i32,
        )
    }

    pub(super) fn source(&self) -> Source<'_> {
        Source::new(self.data, self.width, self.height, 0, 0)
    }

    pub(super) fn target(&mut self) -> Target<'_> {
        Target::new(self.data, self.width, self.height, 0, 0)
    }
}
