/// A rectangle of whole pixels, from `(x0, y0)` inclusive to `(x1, y1)` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IntRect {
    /// The left edge.
    pub x0: i32,
    /// The top edge.
    pub y0: i32,
    /// The right edge.
    pub x1: i32,
    /// The bottom edge.
    pub y1: i32,
}

impl IntRect {
    /// Creates a rectangle from its edges.
    pub const fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    /// The width, or 0 if the rectangle is empty.
    pub fn width(&self) -> u32 {
        (i64::from(self.x1) - i64::from(self.x0)).clamp(0, i64::from(u32::MAX)) as u32
    }

    /// The height, or 0 if the rectangle is empty.
    pub fn height(&self) -> u32 {
        (i64::from(self.y1) - i64::from(self.y0)).clamp(0, i64::from(u32::MAX)) as u32
    }

    /// Whether the rectangle contains no pixels.
    pub fn is_empty(&self) -> bool {
        self.x0 >= self.x1 || self.y0 >= self.y1
    }

    /// The pixels in both rectangles.
    pub fn intersect(&self, other: &Self) -> Self {
        Self::new(
            self.x0.max(other.x0),
            self.y0.max(other.y0),
            self.x1.min(other.x1),
            self.y1.min(other.y1),
        )
    }
}
