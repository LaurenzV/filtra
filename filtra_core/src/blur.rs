//! The decimation of Gaussian blurs.

use crate::IntRect;

/// A Gaussian blur as the passes that apply it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Blur {
    /// The input unchanged.
    Unchanged,
    /// A transparent result.
    Transparent,
    /// The input decimated.
    Decimated {
        /// The passes in execution order.
        passes: Vec<Pass>,
        /// The weights of the Gaussian along x and y from the centre outwards, in units of 2^-15.
        weights: [Vec<u16>; 2],
    },
}

impl Blur {
    /// The blur of `source` into `target` with the standard deviations `std_dev` in pixels.
    pub fn new(std_dev: [f32; 2], source: IntRect, target: IntRect) -> Self {
        let axes = std_dev.map(Axis::new);

        if axes.iter().all(|axis| axis.weights.is_empty()) {
            return Self::Unchanged;
        }

        // The blur is decimated: each axis is halved with [1, 3, 3, 1] / 8 until a variance of at
        // most 4 is left, that rest is a small Gaussian on the coarsest image, and the image is
        // then doubled back with [1, 3] / 4 interpolation. Halving and doubling each add a
        // variance of 0.75 in the finer grid, and a variance in the coarser grid is four times as
        // large in the finer one, so the passes add up to the requested variance. Each level has
        // half the pixels of the one before along every halved axis, which keeps the work per
        // pixel constant for any blur.
        let levels = axes[0].halvings.max(axes[1].halvings);
        let (source, target) = (spans(source), spans(target));
        let layout: [Layout; 2] =
            core::array::from_fn(|a| Layout::new(&axes[a], levels, source[a], target[a]));

        if layout.iter().any(|axis| axis.up[0].is_empty()) {
            return Self::Transparent;
        }

        let resampling = |level: usize, op: Op| {
            [0, 1].map(|a| {
                if level < axes[a].halvings {
                    op
                } else {
                    Op::Copy
                }
            })
        };
        let gauss = |axis: &Axis| {
            if axis.weights.is_empty() {
                Op::Copy
            } else {
                Op::Gauss
            }
        };
        let down = |level: usize| [layout[0].down[level], layout[1].down[level]];
        let up = |level: usize| [layout[0].up[level], layout[1].up[level]];

        let passes = (0..levels)
            .map(|level| (resampling(level, Op::Halve), down(level + 1)))
            .chain([(axes.each_ref().map(gauss), up(levels))])
            .chain(
                (0..levels)
                    .rev()
                    .map(|level| (resampling(level, Op::Double), up(level))),
            )
            .map(|(ops, spans)| Pass { ops, spans })
            .collect();

        Self::Decimated {
            passes,
            weights: axes.map(|axis| axis.weights),
        }
    }
}

/// One pass of a blur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pass {
    /// The operations along x and y.
    pub ops: [Op; 2],
    /// The pixels the pass computes along x and y.
    pub spans: [Span; 2],
}

/// The operation of a pass along one axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// The pixels unchanged.
    Copy,
    /// [1, 3, 3, 1] / 8 to every other pixel.
    Halve,
    /// [1, 3] / 4 and [3, 1] / 4 between the pixels.
    Double,
    /// The Gaussian with the weights of the axis.
    Gauss,
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

struct Axis {
    halvings: usize,
    /// The weight of the centre followed by those of the distances 1, 2, …, summing to 2^15 over
    /// both sides; empty for an axis that isn't blurred.
    weights: Vec<u16>,
}

impl Axis {
    fn new(std_dev: f32) -> Self {
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

/// The pixels each pass computes along one axis.
struct Layout {
    /// The halved levels, indexed by level from 1.
    down: Vec<Span>,
    /// The Gaussian on the coarsest level and the levels doubled from it, indexed by level.
    up: Vec<Span>,
}

impl Layout {
    fn new(axis: &Axis, levels: usize, source: Span, target: Span) -> Self {
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

/// Beyond this, a blur spreads any image too thin to leave a visible pixel.
const MAX_STD_DEV: f32 = 1e11;
