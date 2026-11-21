use core::ops::Range;

use fearless_simd::{Simd, prelude::*, u8x32, u16x16, u32x8};

use crate::dispatch::dispatch;
use crate::mul_q15::MulQ15;
use crate::simd::Wide;
use crate::{IntRect, Source, Target};

/// Halves `src` along the axes with [1, 3, 3, 1] / 8.
pub fn downsample(src: Source<'_>, dst: Target<'_>, axes: [bool; 2]) {
    let ops = axes.map(|axis| if axis { Op::Halve } else { Op::Copy });

    dispatch!(simd => fill_target(simd, src, ops, dst));
}

/// Doubles `src` along the axes with [1, 3] / 4 interpolation.
pub fn upsample(src: Source<'_>, dst: Target<'_>, axes: [bool; 2]) {
    let ops = axes.map(|axis| if axis { Op::Double } else { Op::Copy });

    dispatch!(simd => fill_target(simd, src, ops, dst));
}

/// Convolves `src` with symmetric weights from the centre outwards, in units of 2^-15.
pub fn gaussian_blur(src: Source<'_>, dst: Target<'_>, weights: [&[u16]; 2]) {
    assert!(
        weights
            .iter()
            .all(|weights| weights.len() <= MAX_TAPS / 2 + 1)
    );

    let ops = weights.map(|weights| {
        if weights.is_empty() {
            Op::Copy
        } else {
            Op::Gauss(weights)
        }
    });

    dispatch!(simd => fill_target(simd, src, ops, dst));
}

const CHUNK: usize = 8;
const MAX_TAPS: usize = 13;
/// The fractional bits of a horizontal Gaussian, which leave room for the sum of two of them.
const GAUSS_FRACTION: u32 = 6;

/// Columns or rows from `.0` inclusive to `.1` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span(i64, i64);

impl Span {
    const EMPTY: Span = Span(0, 0);

    fn new(start: i64, end: i64) -> Span {
        if start < end {
            Span(start, end)
        } else {
            Self::EMPTY
        }
    }

    fn len(self) -> usize {
        (self.1 - self.0) as usize
    }

    fn is_empty(self) -> bool {
        self.0 >= self.1
    }

    fn grow(self, by: i64) -> Span {
        self.map(|a, b| (a - by, b + by))
    }

    /// The pixels that doubling this span makes non-zero, which are also the pixels that halving
    /// reads for it.
    fn doubled(self) -> Span {
        self.map(|a, b| (2 * a - 1, 2 * b + 1))
    }

    fn map(self, f: impl FnOnce(i64, i64) -> (i64, i64)) -> Span {
        if self.is_empty() {
            return Self::EMPTY;
        }

        let (a, b) = f(self.0, self.1);
        Span::new(a, b)
    }
}

fn spans(rect: IntRect) -> [Span; 2] {
    let span = |a: i32, b: i32| Span::new(a.into(), b.into());

    [span(rect.x0, rect.x1), span(rect.y0, rect.y1)]
}

/// A horizontal or vertical step of a pass.
#[derive(Clone, Copy)]
enum Op<'a> {
    Copy,
    /// [1, 3, 3, 1] / 8 to every other pixel.
    Halve,
    /// [1, 3] / 4 and [3, 1] / 4 between the pixels.
    Double,
    /// Symmetric weights in 15-bit fixed point, from the centre outwards.
    Gauss(&'a [u16]),
}

impl Op<'_> {
    /// The fractional bits of the horizontal result.
    fn fraction(self) -> u32 {
        match self {
            Op::Copy => 0,
            Op::Halve => 3,
            Op::Double => 2,
            Op::Gauss(_) => GAUSS_FRACTION,
        }
    }

    /// The first input row and the number of rows that output row `y` reads.
    fn rows(self, y: i64) -> (i64, usize) {
        match self {
            Op::Copy => (y, 1),
            Op::Halve => (2 * y - 1, 4),
            Op::Double => (y.div_euclid(2) - 1, 3),
            Op::Gauss(weights) => (y - (weights.len() as i64 - 1), 2 * weights.len() - 1),
        }
    }

    /// The input columns that the outputs from `x0` to `x1` read.
    fn reads(self, x0: i64, x1: i64) -> Span {
        match self {
            Op::Copy => Span(x0, x1),
            Op::Halve => Span(x0, x1).doubled(),
            Op::Double => Span(x0 / 2 - 1, x1 / 2 + 1),
            Op::Gauss(weights) => Span(x0, x1).grow(weights.len() as i64 - 1),
        }
    }
}

fn fill_target<S: MulQ15>(simd: S, src: Source<'_>, ops: [Op<'_>; 2], mut dst: Target<'_>) {
    let [x, _] = spans(dst.rect());
    let mut pass = Pass::new(simd, src, ops, x);

    for (y, row) in dst.rows_mut() {
        pass.row(i64::from(y), row);
    }
}

/// One pass: a horizontal step into 16-bit rows, then a vertical step that combines them.
struct Pass<'a, S: MulQ15> {
    simd: S,
    src: Source<'a>,
    ops: [Op<'a>; 2],
    x: Span,
    /// The column of the first horizontal result.
    first: i64,
    parts: Vec<Part>,
    /// Horizontal results in a ring, with the input row each one holds.
    rows: Vec<Vec<[u16; 32]>>,
    held: Vec<Option<i64>>,
    /// The vertical result of the chunks that don't fit the output whole.
    staging: Vec<[u8; 32]>,
}

/// Chunks of the horizontal step that read the input row in the same way.
struct Part {
    chunks: Range<usize>,
    /// The source columns that the chunks read, padded with transparent pixels; `None` if they
    /// lie inside of the input row.
    copy: Option<Vec<u8>>,
}

impl<'a, S: MulQ15> Pass<'a, S> {
    fn new(simd: S, src: Source<'a>, ops: [Op<'a>; 2], x: Span) -> Self {
        // Doubling makes two outputs, an even and an odd one, per input pixel.
        let (first, group) = match ops[0] {
            Op::Double => (x.0.div_euclid(2) * 2, 2),
            _ => (x.0, 1),
        };
        let groups = ((x.1 - first) as usize).div_ceil(group * CHUNK);
        let chunks = groups * group;
        let ring = ops[1].rows(0).1;

        // Only the chunks near the edges of the source read past them, so only their
        // columns are copied.
        let [source, _] = spans(src.rect());
        let reads = |groups: Range<usize>| {
            let start = first + (groups.start * group * CHUNK) as i64;
            ops[0].reads(start, start + (groups.len() * group * CHUNK) as i64)
        };
        let inside = |g: usize| {
            let reads = reads(g..g + 1);
            reads.0 >= source.0 && reads.1 <= source.1
        };
        let start = (0..groups).find(|&g| inside(g)).unwrap_or(groups);
        let end = (start..groups).find(|&g| !inside(g)).unwrap_or(groups);
        let part = |groups: Range<usize>, copied: bool| Part {
            copy: copied.then(|| vec![0; reads(groups.clone()).len() * 4]),
            chunks: groups.start * group..groups.end * group,
        };

        let parts = [
            part(0..start, true),
            part(start..end, false),
            part(end..groups, true),
        ]
        .into_iter()
        .filter(|part| !part.chunks.is_empty())
        .collect();

        Self {
            simd,
            src,
            ops,
            x,
            first,
            parts,
            rows: vec![vec![[0; 32]; chunks]; ring],
            held: vec![None; ring],
            staging: vec![[0; 32]; chunks],
        }
    }

    fn row(&mut self, y: i64, out: &mut [u8]) {
        let simd = self.simd;
        simd.vectorize(
            #[inline(always)]
            || self.row_inner(y, out),
        );
    }

    #[inline(always)]
    fn row_inner(&mut self, y: i64, out: &mut [u8]) {
        let (top, count) = self.ops[1].rows(y);
        let mut slots = [0; MAX_TAPS];

        for (i, slot) in slots.iter_mut().take(count).enumerate() {
            *slot = self.horizontal(top + i as i64);
        }

        let odd = y.rem_euclid(2) == 1;
        let fraction = self.ops[0].fraction();
        let chunks = self.staging.len();
        // The vertical step writes the chunks that line up with `out` straight into it, and the
        // rest into `staging`.
        let skip = (self.x.0 - self.first) as usize * 4;
        let (direct, whole): (&mut [[u8; 32]], usize) = if skip == 0 {
            let direct = out.as_chunks_mut::<32>().0;
            let whole = direct.len();
            (direct, whole)
        } else {
            (&mut [], 0)
        };
        let targets = [
            (0..whole, direct),
            (whole..chunks, &mut self.staging[..chunks - whole]),
        ];

        for (range, target) in targets {
            let ring = &self.rows;
            let rows: [&[[u16; 32]]; MAX_TAPS] = core::array::from_fn(|i| {
                if i < count {
                    &ring[slots[i]][range.clone()]
                } else {
                    &[]
                }
            });
            vertical(
                self.simd,
                self.ops[1],
                fraction,
                &rows[..count],
                odd,
                target,
            );
        }

        let rest = &mut out[whole * 32..];
        rest.copy_from_slice(&self.staging.as_flattened()[skip..skip + rest.len()]);
    }

    /// The ring slot that holds the horizontal result of input row `y`.
    #[inline(always)]
    fn horizontal(&mut self, y: i64) -> usize {
        let slot = y.rem_euclid(self.rows.len() as i64) as usize;

        if self.held[slot] == Some(y) {
            return slot;
        }

        for part in &mut self.parts {
            let first = self.first + (part.chunks.start * CHUNK) as i64;
            let reads = self.ops[0].reads(first, first + (part.chunks.len() * CHUNK) as i64);
            let rect = self.src.rect();
            let x0 = reads.0.max(rect.x0.into());
            let x1 = reads.1.min(rect.x1.into());
            let row = i32::try_from(y)
                .ok()
                .filter(|_| x0 < x1)
                .and_then(|y| self.src.row(y, x0 as i32, x1 as i32));
            let row = row.map(|row| match &mut part.copy {
                Some(copy) => {
                    let start = (x0 - reads.0) as usize * 4;
                    copy[start..start + row.len()].copy_from_slice(row);
                    (copy.as_slice(), reads.0)
                }
                None => (row, reads.0),
            });

            horizontal(
                self.simd,
                self.ops[0],
                row,
                first,
                &mut self.rows[slot][part.chunks.clone()],
            );
        }

        self.held[slot] = Some(y);

        slot
    }
}

/// The horizontal step for the chunks of `out` from column `first`, each an even and an odd
/// [`Wide`] half.
#[inline(always)]
fn horizontal<S: MulQ15>(
    simd: S,
    op: Op<'_>,
    row: Option<(&[u8], i64)>,
    first: i64,
    out: &mut [[u16; 32]],
) {
    let Some((row, origin)) = row else {
        out.fill([0; 32]);
        return;
    };

    // Every chunk reads a fixed window of the row at fixed offsets, which needs no bounds checks.
    let reads = op.reads(first, first + (out.len() * CHUNK) as i64);
    let row = &row[(reads.0 - origin) as usize * 4..(reads.1 - origin) as usize * 4];
    let load = {
        #[inline(always)]
        |bytes: &[u8]| u8x32::from_slice(simd, &bytes[..CHUNK * 4])
    };

    match op {
        Op::Copy => {
            for (out, bytes) in out.iter_mut().zip(row.as_chunks::<{ CHUNK * 4 }>().0) {
                store_wide(out, Wide::from_pixels(load(bytes)));
            }
        }
        Op::Halve => {
            let windows = row.windows((2 * CHUNK + 2) * 4).step_by(2 * CHUNK * 4);

            for (out, window) in out.iter_mut().zip(windows) {
                let pixels = {
                    #[inline(always)]
                    |pixel: usize| load(&window[pixel * 4..]).bitcast::<u32x8<S>>()
                };
                let (before, even) = simd.deinterleave_u32x8(pixels(0), pixels(CHUNK));
                let (odd, after) = simd.deinterleave_u32x8(pixels(2), pixels(CHUNK + 2));
                let before = Wide::from_pixels(before.bitcast());
                let even = Wide::from_pixels(even.bitcast());
                let odd = Wide::from_pixels(odd.bitcast());
                let after = Wide::from_pixels(after.bitcast());
                let taps = {
                    #[inline(always)]
                    |a: u16x16<S>, b: u16x16<S>, c: u16x16<S>, d: u16x16<S>| a + d + times3(b + c)
                };

                store_wide(
                    out,
                    Wide {
                        even: taps(before.even, even.even, odd.even, after.even),
                        odd: taps(before.odd, even.odd, odd.odd, after.odd),
                    },
                );
            }
        }
        Op::Double => {
            let pairs = out.as_chunks_mut::<2>().0;
            let windows = row.windows((CHUNK + 2) * 4).step_by(CHUNK * 4);

            for (out, window) in pairs.iter_mut().zip(windows) {
                let wide = {
                    #[inline(always)]
                    |pixel: usize| Wide::from_pixels(load(&window[pixel * 4..]))
                };
                let (left, centre, right) = (wide(0), wide(1), wide(2));
                let half = {
                    #[inline(always)]
                    |left: u16x16<S>, centre: u16x16<S>, right: u16x16<S>| {
                        let centre = times3(centre);
                        let (even, odd) = ((left + centre).bitcast(), (right + centre).bitcast());
                        let (low, high): (u32x8<S>, u32x8<S>) = simd.interleave_u32x8(even, odd);
                        (low.bitcast::<u16x16<S>>(), high.bitcast::<u16x16<S>>())
                    }
                };
                let (even_low, even_high) = half(left.even, centre.even, right.even);
                let (odd_low, odd_high) = half(left.odd, centre.odd, right.odd);
                let [low, high] = out;

                store_wide(
                    low,
                    Wide {
                        even: even_low,
                        odd: odd_low,
                    },
                );
                store_wide(
                    high,
                    Wide {
                        even: even_high,
                        odd: odd_high,
                    },
                );
            }
        }
        Op::Gauss(weights) => gauss_horizontal(simd, weights.len() - 1, weights, row, out),
    }
}

/// The horizontal Gaussian of radius `r` for the chunks of `out`, from `row` starting `r` pixels
/// before the first output.
#[inline(always)]
fn gauss_horizontal<S: MulQ15>(
    simd: S,
    r: usize,
    weights: &[u16],
    row: &[u8],
    out: &mut [[u16; 32]],
) {
    let weights = fixed_weights(weights);
    let windows = row.windows((CHUNK + 2 * r) * 4).step_by(CHUNK * 4);

    for (out, window) in out.iter_mut().zip(windows) {
        let wide = {
            #[inline(always)]
            |pixel: usize| {
                Wide::from_pixels(u8x32::from_slice(simd, &window[pixel * 4..][..CHUNK * 4]))
            }
        };
        // Every product is rounded, so the pixels get fractional bits first to keep that small.
        let term = {
            #[inline(always)]
            |v: u16x16<S>, weight: u16| weighted(simd, v << GAUSS_FRACTION, weight)
        };
        let centre = wide(r);
        let mut even = term(centre.even, weights[0]);
        let mut odd = term(centre.odd, weights[0]);

        for (i, &weight) in weights.iter().enumerate().take(r + 1).skip(1) {
            let (a, b) = (wide(r + i), wide(r - i));
            even += term(a.even + b.even, weight);
            odd += term(a.odd + b.odd, weight);
        }

        store_wide(out, Wide { even, odd });
    }
}

/// The vertical step into the chunks of `out`, from horizontal results with `fraction` fractional
/// bits.
#[inline(always)]
fn vertical<S: MulQ15>(
    simd: S,
    op: Op<'_>,
    fraction: u32,
    rows: &[&[[u16; 32]]],
    odd: bool,
    out: &mut [[u8; 32]],
) {
    let even_odd = {
        #[inline(always)]
        |chunk: &[u16; 32]| {
            (
                u16x16::from_slice(simd, &chunk[..16]),
                u16x16::from_slice(simd, &chunk[16..]),
            )
        }
    };
    let store = {
        #[inline(always)]
        |out: &mut [u8; 32], even: u16x16<S>, odd: u16x16<S>| {
            Wide { even, odd }.store(out);
        }
    };

    match op {
        Op::Copy => {
            for (out, a) in out.iter_mut().zip(rows[0]) {
                let (even, odd) = even_odd(a);
                store(out, round(even, fraction), round(odd, fraction));
            }
        }
        Op::Halve => {
            let rows = rows[0].iter().zip(rows[1]).zip(rows[2].iter().zip(rows[3]));

            for (out, ((a, b), (c, d))) in out.iter_mut().zip(rows) {
                let [a, b, c, d] = [even_odd(a), even_odd(b), even_odd(c), even_odd(d)];
                let taps = {
                    #[inline(always)]
                    |a: u16x16<S>, b: u16x16<S>, c: u16x16<S>, d: u16x16<S>| {
                        round(a + d + times3(b + c), fraction + 3)
                    }
                };

                store(out, taps(a.0, b.0, c.0, d.0), taps(a.1, b.1, c.1, d.1));
            }
        }
        Op::Double => {
            // An even row is a quarter of the row above and three quarters of the one below
            // it, an odd row three quarters of that one and a quarter of the next.
            let (outer, inner) = if odd {
                (rows[2], rows[1])
            } else {
                (rows[0], rows[1])
            };

            for (out, (a, b)) in out.iter_mut().zip(outer.iter().zip(inner)) {
                let (a, b) = (even_odd(a), even_odd(b));
                let tap = {
                    #[inline(always)]
                    |a: u16x16<S>, b: u16x16<S>| round(a + times3(b), fraction + 2)
                };

                store(out, tap(a.0, b.0), tap(a.1, b.1));
            }
        }
        Op::Gauss(weights) => gauss_vertical(simd, weights.len() - 1, weights, fraction, rows, out),
    }
}

/// The vertical Gaussian of radius `r` into the chunks of `out`, from the `2r + 1` rows around it.
#[inline(always)]
fn gauss_vertical<S: MulQ15>(
    simd: S,
    r: usize,
    weights: &[u16],
    fraction: u32,
    rows: &[&[[u16; 32]]],
    out: &mut [[u8; 32]],
) {
    let weights = fixed_weights(weights);
    let scale = GAUSS_FRACTION - fraction;
    let len = out.len();
    // Rows of exactly the output length let the compiler drop the bounds checks.
    let centre = &rows[r][..len];
    let empty: &[[u16; 32]] = &[];
    let above: [&[[u16; 32]]; MAX_TAPS / 2] = core::array::from_fn(|i| {
        if i < r {
            &rows[r - 1 - i][..len]
        } else {
            empty
        }
    });
    let below: [&[[u16; 32]]; MAX_TAPS / 2] = core::array::from_fn(|i| {
        if i < r {
            &rows[r + 1 + i][..len]
        } else {
            empty
        }
    });

    for (chunk, (out, centre)) in out.iter_mut().zip(centre).enumerate() {
        let sum = {
            #[inline(always)]
            |half: usize| {
                let load = {
                    #[inline(always)]
                    |row: &[u16; 32]| u16x16::from_slice(simd, &row[half * 16..][..16])
                };
                let mut sum = weighted(simd, load(centre) << scale, weights[0]);

                for (i, (above, below)) in above.iter().zip(&below).enumerate().take(r) {
                    let pair = load(&above[chunk]) + load(&below[chunk]);
                    sum += weighted(simd, pair << scale, weights[i + 1]);
                }

                round(sum, GAUSS_FRACTION)
            }
        };

        Wide {
            even: sum(0),
            odd: sum(1),
        }
        .store(out);
    }
}

/// The weights from the centre outwards in an array of the largest size, padded with zeros.
#[inline(always)]
fn fixed_weights(weights: &[u16]) -> [u16; MAX_TAPS / 2 + 1] {
    core::array::from_fn(|i| weights.get(i).copied().unwrap_or(0))
}

/// `v` times the Q15 `weight`, rounded.
#[inline(always)]
fn weighted<S: MulQ15>(simd: S, v: u16x16<S>, weight: u16) -> u16x16<S> {
    simd.mul_q15(v.bitcast(), simd.splat_i16x16(weight as i16))
        .bitcast()
}

#[inline(always)]
fn store_wide<S: Simd>(out: &mut [u16; 32], wide: Wide<S>) {
    wide.even.store_slice(&mut out[..16]);
    wide.odd.store_slice(&mut out[16..]);
}

#[inline(always)]
fn times3<S: Simd>(v: u16x16<S>) -> u16x16<S> {
    v + (v << 1)
}

#[inline(always)]
fn round<S: Simd>(v: u16x16<S>, bits: u32) -> u16x16<S> {
    (v + ((1 << bits) >> 1)) >> bits
}
