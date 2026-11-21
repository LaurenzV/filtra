//! The decimation of Gaussian blurs.

use crate::IntRect;

/// The halvings and the remaining Gaussian of one axis of a blur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Axis {
    /// The number of times the axis is halved.
    pub halvings: usize,
    /// The weights of the remaining Gaussian from the centre outwards, in units of 2^-15.
    pub weights: Vec<u16>,
}

impl Axis {
    /// The decimation of an axis with the standard deviation `std_dev` in pixels.
    pub fn new(std_dev: f32) -> Self {
        if std_dev.is_nan() || std_dev <= 0.0 {
            return Self {
                halvings: 0,
                weights: Vec::new(),
            };
        }

        let std_dev = f64::from(std_dev.min(MAX_STD_DEV));
        let mut variance = std_dev * std_dev;
        let mut halvings = 0;

        while variance > 4.0 {
            variance = (variance - 1.5) / 4.0;
            halvings += 1;
        }

        let radius = (3.0 * variance.sqrt()).ceil() as usize;
        let exact: Vec<f64> = (0..=radius)
            .map(|i| (-((i * i) as f64) / (2.0 * variance)).exp())
            .collect();
        let total = exact[0] + 2.0 * exact[1..].iter().sum::<f64>();
        let mut weights: Vec<u16> = exact
            .iter()
            .map(|w| (w / total * 32768.0).round() as u16)
            .collect();
        let sum =
            u32::from(weights[0]) + 2 * weights[1..].iter().map(|&w| u32::from(w)).sum::<u32>();
        // The rounding error goes to the centre. A lone centre weight of 2^15 doesn't fit the
        // 15-bit range, and 2^15 - 1 leaves every 8-bit value unchanged.
        weights[0] = (u32::from(weights[0]) + 32768 - sum).min(32767) as u16;

        Self { halvings, weights }
    }
}

/// The pixels each pass of a blur computes along one axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The halved levels, indexed by level from 1.
    pub down: Vec<Span>,
    /// The Gaussian on the coarsest level and the levels doubled from it, indexed by level.
    pub up: Vec<Span>,
}

impl Layout {
    /// The spans of an axis with `levels` halvings that blurs `source` into `target`.
    pub fn new(axis: &Axis, levels: usize, source: Span, target: Span) -> Self {
        let resampled = |level: usize, span: Span, resample: fn(Span) -> Span| {
            if level < axis.halvings {
                resample(span)
            } else {
                span
            }
        };
        let mut data = vec![source];
        let mut needed = vec![target];

        for level in 0..levels {
            data.push(resampled(level, data[level], Span::halved));
            needed.push(resampled(level, needed[level], Span::doubling_reads));
        }

        let radius = axis.weights.len().saturating_sub(1) as i64;
        let gauss = needed[levels].intersect(data[levels].grow(radius));
        let mut down = vec![Span::EMPTY; levels + 1];

        if levels > 0 {
            down[levels] = gauss.grow(radius).intersect(data[levels]);
        }

        for level in (1..levels).rev() {
            let read = resampled(level, down[level + 1], Span::doubled);
            down[level] = read.intersect(data[level]);
        }

        let mut up = vec![gauss; levels + 1];

        for level in (0..levels).rev() {
            let spread = resampled(level, up[level + 1], Span::doubled);
            up[level] = needed[level].intersect(spread);
        }

        Self { down, up }
    }
}

/// Columns or rows from `.0` inclusive to `.1` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span(pub i64, pub i64);

impl Span {
    /// The span without pixels.
    pub const EMPTY: Span = Span(0, 0);

    /// The span from `start` to `end`.
    pub fn new(start: i64, end: i64) -> Span {
        if start < end {
            Span(start, end)
        } else {
            Self::EMPTY
        }
    }

    /// The number of pixels.
    pub fn len(self) -> usize {
        (self.1 - self.0) as usize
    }

    /// Whether the span has no pixels.
    pub fn is_empty(self) -> bool {
        self.0 >= self.1
    }

    /// Whether the span contains pixel `i`.
    pub fn contains(self, i: i64) -> bool {
        (self.0..self.1).contains(&i)
    }

    /// The pixels in both spans.
    pub fn intersect(self, other: Span) -> Span {
        Span::new(self.0.max(other.0), self.1.min(other.1))
    }

    /// The span extended by `by` pixels on both sides.
    pub fn grow(self, by: i64) -> Span {
        self.map(|a, b| (a - by, b + by))
    }

    /// The pixels that halving this span makes non-zero.
    pub fn halved(self) -> Span {
        self.map(|a, b| ((a - 1).div_euclid(2), b.div_euclid(2) + 1))
    }

    /// The pixels that doubling this span makes non-zero, and that halving reads for it.
    pub fn doubled(self) -> Span {
        self.map(|a, b| (2 * a - 1, 2 * b + 1))
    }

    /// The coarser pixels that doubling into this span reads.
    pub fn doubling_reads(self) -> Span {
        self.map(|a, b| (a.div_euclid(2) - 1, (b - 1).div_euclid(2) + 2))
    }

    fn map(self, f: impl FnOnce(i64, i64) -> (i64, i64)) -> Span {
        if self.is_empty() {
            return Self::EMPTY;
        }

        let (a, b) = f(self.0, self.1);
        Span::new(a, b)
    }
}

/// The columns and rows of `rect`.
pub fn spans(rect: IntRect) -> [Span; 2] {
    let span = |a: i32, b: i32| Span::new(a.into(), b.into());
    [span(rect.x0, rect.x1), span(rect.y0, rect.y1)]
}

// Beyond this, a blur spreads any image too thin to leave a visible pixel.
const MAX_STD_DEV: f32 = 1e11;
