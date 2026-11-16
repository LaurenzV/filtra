use fearless_simd::{Level, Simd, dispatch, prelude::*, u8x32, u32x8};

use crate::simd::for_each_chunk;
use crate::{Pixmap, Source, Target};

/// Whether [`morphology`] takes the minimum or the maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MorphologyOperator {
    /// The component-wise minimum (`erode`).
    Erode,
    /// The component-wise maximum (`dilate`).
    Dilate,
}

/// Sets each pixel to the component-wise minimum or maximum of `src` over the
/// `2 rx + 1` by `2 ry + 1` pixels centered on it.
pub fn morphology(
    src: Source<'_>,
    dst: Target<'_>,
    operator: MorphologyOperator,
    rx: u32,
    ry: u32,
) {
    let level = Level::new();

    match operator {
        MorphologyOperator::Erode => {
            dispatch!(level, simd => morphology_inner::<_, false>(simd, src, dst, rx, ry));
        }
        MorphologyOperator::Dilate => {
            dispatch!(level, simd => morphology_inner::<_, true>(simd, src, dst, rx, ry));
        }
    }
}

const BAND: usize = 8;

#[inline(always)]
fn morphology_inner<S: Simd, const DILATE: bool>(
    simd: S,
    src: Source<'_>,
    dst: Target<'_>,
    rx: u32,
    ry: u32,
) {
    let (rect, source) = (dst.rect(), src.rect());
    let x0 = (i64::from(rect.x0) - i64::from(rx)).max(i64::from(source.x0));
    let x1 = (i64::from(rect.x1) + i64::from(rx)).min(i64::from(source.x1));
    let (x0, x1) = (x0 as i32, x1.max(x0) as i32);
    let mut columns = Pixmap::new((x1 - x0) as u32, rect.height());
    let mut scratch = Vec::new();

    vertical::<S, DILATE>(simd, src, columns.target(x0, rect.y0), ry, &mut scratch);
    horizontal::<S, DILATE>(simd, columns.source(x0, rect.y0), dst, rx, &mut scratch);
}

#[inline(always)]
fn vertical<S: Simd, const DILATE: bool>(
    simd: S,
    src: Source<'_>,
    mut dst: Target<'_>,
    r: u32,
    scratch: &mut Vec<u8>,
) {
    let (rect, source) = (dst.rect(), src.rect());
    let first = i64::from(rect.y0) - i64::from(r);
    let rows = |y: i64| src.row(y as i32, rect.x0, rect.x1).unwrap();
    let present = (i64::from(source.y0), i64::from(source.y1));
    let len = rect.width() as usize * 4;
    let outputs = dst.rows_mut().map(|(_, row)| row);

    sliding::<S, DILATE>(simd, rows, present, first, r, len, scratch, outputs);
}

#[inline(always)]
fn horizontal<S: Simd, const DILATE: bool>(
    simd: S,
    src: Source<'_>,
    mut dst: Target<'_>,
    r: u32,
    scratch: &mut Vec<u8>,
) {
    let (rect, source) = (dst.rect(), src.rect());
    let width = rect.width() as usize;
    let first = i64::from(rect.x0) - i64::from(r);
    let mut columns = vec![0; source.width() as usize * BAND * 4];
    let mut results = vec![0; width * BAND * 4];
    let mut rows = dst.rows_mut();

    loop {
        let band: [Option<(i32, &mut [u8])>; BAND] = core::array::from_fn(|_| rows.next());

        let Some(y) = band[0].as_ref().map(|(y, _)| *y) else {
            return;
        };

        let inputs: [Option<&[u8]>; BAND] = core::array::from_fn(|k| {
            band[k]
                .as_ref()
                .and_then(|_| src.row(y + k as i32, source.x0, source.x1))
        });
        transpose(simd, inputs, &mut columns);

        let column =
            |x: i64| &columns[(x - i64::from(source.x0)) as usize * BAND * 4..][..BAND * 4];
        let present = (i64::from(source.x0), i64::from(source.x1));
        let outputs = results.as_chunks_mut::<{ BAND * 4 }>().0.iter_mut();
        let outputs = outputs.map(|column| column.as_mut_slice());
        sliding::<S, DILATE>(simd, column, present, first, r, BAND * 4, scratch, outputs);

        let mut band = band.map(|row| row.map(|(_, row)| row));
        untranspose(simd, &results, &mut band);
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn sliding<'i, 'o, S: Simd, const DILATE: bool>(
    simd: S,
    rows: impl Fn(i64) -> &'i [u8],
    present: (i64, i64),
    first: i64,
    r: u32,
    len: usize,
    scratch: &mut Vec<u8>,
    outputs: impl Iterator<Item = &'o mut [u8]>,
) {
    // van Herk/Gil-Werman: each window is a suffix of one block of `2 r + 1` rows combined with
    // a prefix of the next, so the cost doesn't depend on `r`. Only suffixes starting at present
    // rows are stored, since the others are transparent or equal to the first stored one, which
    // bounds the memory by the input instead of the radius.
    let window = 2 * i64::from(r) + 1;
    let row = |i: i64| (present.0..present.1).contains(&i).then(|| rows(i));
    let mut outputs = outputs.peekable();
    let mut block = first;

    while outputs.peek().is_some() {
        let (lo, hi) = (block.max(present.0), (block + window).min(present.1));
        let stored = (hi - lo).max(0) as usize;

        scratch.resize((stored + 1) * len, 0);

        let (suffixes, rest) = scratch.split_at_mut(stored * len);
        let prefix = &mut rest[..len];

        for k in (0..stored).rev() {
            let (head, tail) = suffixes.split_at_mut((k + 1) * len);
            let output = &mut head[k * len..];
            let current = rows(lo + k as i64);

            if k + 1 < stored {
                combine::<S, DILATE>(simd, Some(current), Some(&tail[..len]), output);
            } else if hi < block + window {
                combine::<S, DILATE>(simd, Some(current), None, output);
            } else {
                output.copy_from_slice(current);
            }
        }

        let suffix = |i: i64| {
            let transparent = i >= hi || stored == 0 || (i < lo && !DILATE);
            (!transparent).then(|| &suffixes[(i.max(lo) - lo) as usize * len..][..len])
        };
        let mut covered = Prefix::Empty;

        for j in 0..window {
            let Some(output) = outputs.next() else {
                return;
            };

            if j > 0 {
                covered = match (row(block + window + j - 1), covered) {
                    (Some(current), Prefix::Rows) => {
                        accumulate::<S, DILATE>(simd, Some(current), prefix);
                        Prefix::Rows
                    }
                    (Some(current), Prefix::Empty) => {
                        prefix.copy_from_slice(current);
                        Prefix::Rows
                    }
                    (_, Prefix::Transparent) => Prefix::Transparent,
                    (None, covered) if DILATE => covered,
                    (None, _) => Prefix::Transparent,
                };
            }

            match covered {
                Prefix::Empty => assign(suffix(block + j), output),
                Prefix::Rows => combine::<S, DILATE>(simd, suffix(block + j), Some(prefix), output),
                Prefix::Transparent => output.fill(0),
            }
        }

        block += window;
    }
}

#[derive(Clone, Copy)]
enum Prefix {
    Empty,
    Rows,
    Transparent,
}

fn assign(row: Option<&[u8]>, output: &mut [u8]) {
    match row {
        Some(row) => output.copy_from_slice(row),
        None => output.fill(0),
    }
}

#[inline(always)]
fn accumulate<S: Simd, const DILATE: bool>(simd: S, row: Option<&[u8]>, output: &mut [u8]) {
    for_each_chunk([row], output, {
        #[inline(always)]
        |[row]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
            let current = u8x32::from_slice(simd, output);
            apply::<S, DILATE>(current, u8x32::from_slice(simd, row)).store_slice(output);
        }
    });
}

#[inline(always)]
fn combine<S: Simd, const DILATE: bool>(
    simd: S,
    a: Option<&[u8]>,
    b: Option<&[u8]>,
    output: &mut [u8],
) {
    for_each_chunk([a, b], output, {
        #[inline(always)]
        |[a, b]: [&[u8; 32]; 2], output: &mut [u8; 32]| {
            let (a, b) = (u8x32::from_slice(simd, a), u8x32::from_slice(simd, b));
            apply::<S, DILATE>(a, b).store_slice(output);
        }
    });
}

#[inline(always)]
fn apply<S: Simd, const DILATE: bool>(a: u8x32<S>, b: u8x32<S>) -> u8x32<S> {
    if DILATE { a.max(b) } else { a.min(b) }
}

#[inline(always)]
fn transpose<S: Simd>(simd: S, rows: [Option<&[u8]>; BAND], columns: &mut [u8]) {
    let width = columns.len() / (BAND * 4);
    let full = width / BAND * BAND;

    for x in (0..full).step_by(BAND) {
        let mut vectors = [u32x8::splat(simd, 0); BAND];

        for (vector, row) in vectors.iter_mut().zip(rows) {
            if let Some(row) = row {
                *vector = u8x32::from_slice(simd, &row[x * 4..][..32]).bitcast();
            }
        }

        for (k, vector) in transpose8(vectors).into_iter().enumerate() {
            let column = &mut columns[(x + k) * BAND * 4..][..32];
            vector.bitcast::<u8x32<S>>().store_slice(column);
        }
    }

    for x in full..width {
        for (k, row) in rows.iter().enumerate() {
            let pixel = &mut columns[(x * BAND + k) * 4..][..4];
            assign(row.map(|row| &row[x * 4..][..4]), pixel);
        }
    }
}

#[inline(always)]
fn untranspose<S: Simd>(simd: S, columns: &[u8], rows: &mut [Option<&mut [u8]>; BAND]) {
    let width = columns.len() / (BAND * 4);
    let full = width / BAND * BAND;

    for x in (0..full).step_by(BAND) {
        let mut vectors = [u32x8::splat(simd, 0); BAND];

        for (k, vector) in vectors.iter_mut().enumerate() {
            let column = &columns[(x + k) * BAND * 4..][..32];
            *vector = u8x32::from_slice(simd, column).bitcast();
        }

        for (row, vector) in rows.iter_mut().zip(transpose8(vectors)) {
            if let Some(row) = row {
                vector
                    .bitcast::<u8x32<S>>()
                    .store_slice(&mut row[x * 4..][..32]);
            }
        }
    }

    for x in full..width {
        for (k, row) in rows.iter_mut().enumerate() {
            if let Some(row) = row {
                row[x * 4..][..4].copy_from_slice(&columns[(x * BAND + k) * 4..][..4]);
            }
        }
    }
}

#[inline(always)]
fn transpose8<S: Simd>(vectors: [u32x8<S>; 8]) -> [u32x8<S>; 8] {
    // Each round rotates the 6 bits of (vector, lane) by one, so three swap them.
    let mut vectors = vectors;

    for _ in 0..3 {
        let mut next = vectors;

        for i in 0..4 {
            let (low, high) = vectors[i].interleave(vectors[i + 4]);
            next[2 * i] = low;
            next[2 * i + 1] = high;
        }

        vectors = next;
    }

    vectors
}
