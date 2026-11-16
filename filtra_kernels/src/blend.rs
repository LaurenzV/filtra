use fearless_simd::{Level, Select, Simd, dispatch, f32x8, prelude::*, u16x16, u32x8};

use crate::pixmap::segments;
use crate::simd::{Wide, div255, for_each_chunk, pack, unpack, unpremultiply};
use crate::{Source, Target};

/// A blend mode from Compositing and Blending Level 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// `normal`.
    Normal,
    /// `multiply`.
    Multiply,
    /// `screen`.
    Screen,
    /// `overlay`.
    Overlay,
    /// `darken`.
    Darken,
    /// `lighten`.
    Lighten,
    /// `color-dodge`.
    ColorDodge,
    /// `color-burn`.
    ColorBurn,
    /// `hard-light`.
    HardLight,
    /// `soft-light`.
    SoftLight,
    /// `difference`.
    Difference,
    /// `exclusion`.
    Exclusion,
    /// `hue`.
    Hue,
    /// `saturation`.
    Saturation,
    /// `color`.
    Color,
    /// `luminosity`.
    Luminosity,
}

/// Blends `src` into `backdrop`.
pub fn blend(
    src: Source<'_>,
    backdrop: Source<'_>,
    mode: BlendMode,
    composite: bool,
    dst: Target<'_>,
) {
    let level = Level::new();

    // One dispatch per mode keeps each mode in its own function, which keeps the stack frames of
    // debug builds small.
    macro_rules! blend_with {
        ($mode:ident) => {
            dispatch!(level, simd => blend_rows(simd, src, backdrop, composite, dst, {
                #[inline(always)]
                |simd, inputs, composite, output| $mode(simd, inputs, composite, output)
            }))
        };
    }

    match mode {
        BlendMode::Normal => blend_with!(normal),
        BlendMode::Multiply => blend_with!(multiply),
        BlendMode::Screen => blend_with!(screen),
        BlendMode::Overlay => blend_with!(overlay),
        BlendMode::Darken => blend_with!(darken),
        BlendMode::Lighten => blend_with!(lighten),
        BlendMode::ColorDodge => blend_with!(color_dodge),
        BlendMode::ColorBurn => blend_with!(color_burn),
        BlendMode::HardLight => blend_with!(hard_light),
        BlendMode::SoftLight => blend_with!(soft_light),
        BlendMode::Difference => blend_with!(difference),
        BlendMode::Exclusion => blend_with!(exclusion),
        BlendMode::Hue => blend_with!(hue),
        BlendMode::Saturation => blend_with!(saturation),
        BlendMode::Color => blend_with!(color),
        BlendMode::Luminosity => blend_with!(luminosity),
    }
}

#[inline(always)]
fn blend_rows<S: Simd>(
    simd: S,
    src: Source<'_>,
    backdrop: Source<'_>,
    composite: bool,
    mut dst: Target<'_>,
    mode: impl Fn(S, [Option<&[u8]>; 2], bool, &mut [u8]),
) {
    let rect = dst.rect();

    for (y, row) in dst.rows_mut() {
        for (x0, x1) in segments(rect.x0, rect.x1, [src.rect(), backdrop.rect()].into_iter()) {
            let output = &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4];
            let inputs = [src.row(y, x0, x1), backdrop.row(y, x0, x1)];

            mode(simd, inputs, composite, output);
        }
    }
}

macro_rules! integer_mode {
    ($name:ident, |$cs:ident, $sa:ident, $cb:ident, $ba:ident| $mixed:expr) => {
        #[inline(always)]
        fn $name<S: Simd>(simd: S, inputs: [Option<&[u8]>; 2], composite: bool, output: &mut [u8]) {
            integer(simd, inputs, composite, output, {
                #[inline(always)]
                #[allow(unused_variables, reason = "Not every mode uses every input.")]
                |$cs, $sa, $cb, $ba| $mixed
            });
        }
    };
}

macro_rules! float_mode {
    ($name:ident, |$s:ident, $b:ident| $mixed:expr) => {
        #[inline(always)]
        fn $name<S: Simd>(simd: S, inputs: [Option<&[u8]>; 2], composite: bool, output: &mut [u8]) {
            float(simd, inputs, composite, output, {
                #[inline(always)]
                |$s, $b| $mixed
            });
        }
    };
}

integer_mode!(normal, |cs, sa, cb, ba| cs * ba);
integer_mode!(multiply, |cs, sa, cb, ba| cs * cb);
integer_mode!(screen, |cs, sa, cb, ba| cs * ba + cb * sa - cs * cb);
integer_mode!(overlay, |cs, sa, cb, ba| mixed_hard_light(cb, ba, cs, sa));
integer_mode!(darken, |cs, sa, cb, ba| (cs * ba).min(cb * sa));
integer_mode!(lighten, |cs, sa, cb, ba| (cs * ba).max(cb * sa));
integer_mode!(hard_light, |cs, sa, cb, ba| mixed_hard_light(
    cs, sa, cb, ba
));
integer_mode!(difference, |cs, sa, cb, ba| {
    let (s, b) = (cs * ba, cb * sa);
    s.max(b) - s.min(b)
});
integer_mode!(exclusion, |cs, sa, cb, ba| cs * ba + cb * sa
    - ((cs * cb) << 1));
float_mode!(color_dodge, |s, b| [
    dodge(s[0], s[3], b[0], b[3]),
    dodge(s[1], s[3], b[1], b[3]),
    dodge(s[2], s[3], b[2], b[3]),
]);
float_mode!(color_burn, |s, b| [
    burn(s[0], s[3], b[0], b[3]),
    burn(s[1], s[3], b[1], b[3]),
    burn(s[2], s[3], b[2], b[3]),
]);
float_mode!(soft_light, |s, b| {
    let both = s[3] * b[3];
    let (s, b) = (unpremultiply(s), unpremultiply(b));
    [
        soft(s[0], b[0]) * both,
        soft(s[1], b[1]) * both,
        soft(s[2], b[2]) * both,
    ]
});
float_mode!(hue, |s, b| {
    let (s, b, bound) = scaled(s, b);
    set_lum(set_sat(s, sat(b)), lum(b), bound)
});
float_mode!(saturation, |s, b| {
    let (s, b, bound) = scaled(s, b);
    set_lum(set_sat(b, sat(s)), lum(b), bound)
});
float_mode!(color, |s, b| {
    let (s, b, bound) = scaled(s, b);
    set_lum(s, lum(b), bound)
});
float_mode!(luminosity, |s, b| {
    let (s, b, bound) = scaled(s, b);
    set_lum(b, lum(s), bound)
});

#[inline(always)]
fn integer<S: Simd>(
    simd: S,
    inputs: [Option<&[u8]>; 2],
    composite: bool,
    output: &mut [u8],
    mixed: impl Fn(u16x16<S>, u16x16<S>, u16x16<S>, u16x16<S>) -> u16x16<S> + Copy,
) {
    let backdrop = u16x16::splat(simd, if composite { u16::MAX } else { 0 });
    let alpha: u16x16<S> = u32x8::splat(simd, 0xffff_0000).bitcast();
    let blended = {
        #[inline(always)]
        |cs: u16x16<S>, sa: u16x16<S>, cb: u16x16<S>, ba: u16x16<S>, mixed: u16x16<S>| {
            // The result is at most 255²·αo, so wrapping arithmetic gives it exactly even where a
            // term overflows.
            div255(cs * (255 - ba) + mixed + ((cb * (255 - sa)) & backdrop))
        }
    };

    for_each_chunk(inputs, output, {
        #[inline(always)]
        |[s, b]: [&[u8; 32]; 2], output: &mut [u8; 32]| {
            let (s, b) = (Wide::load(simd, s), Wide::load(simd, b));
            let (sa, ba) = (s.alpha(), b.alpha());
            // With `αs·αb` as its mixed value, the alpha channel follows the same formula as the
            // colors.
            let odd = (mixed(s.odd, sa, b.odd, ba) & !alpha) | ((sa * ba) & alpha);

            Wide {
                even: blended(s.even, sa, b.even, ba, mixed(s.even, sa, b.even, ba)),
                odd: blended(s.odd, sa, b.odd, ba, odd),
            }
            .store(output);
        }
    });
}

#[inline(always)]
fn float<S: Simd>(
    simd: S,
    inputs: [Option<&[u8]>; 2],
    composite: bool,
    output: &mut [u8],
    mixed: impl Fn([f32x8<S>; 4], [f32x8<S>; 4]) -> [f32x8<S>; 3] + Copy,
) {
    let backdrop = if composite { 1.0 } else { 0.0 };

    for_each_chunk(inputs, output, {
        #[inline(always)]
        |[s, b]: [&[u8; 32]; 2], output: &mut [u8; 32]| {
            let (s, b) = (unpack(simd, s), unpack(simd, b));
            let mixed = mixed(s, b);
            let both = s[3] * b[3];
            let source = 255.0 - b[3];
            let backdrop = (255.0 - s[3]) * backdrop;
            let channel = {
                #[inline(always)]
                |i: usize| {
                    let mixed = mixed[i].max(0.0).min(both);
                    s[i].mul_add(source, b[i].mul_add(backdrop, mixed)) * (1.0 / 255.0)
                }
            };

            pack(
                [
                    channel(0),
                    channel(1),
                    channel(2),
                    (b[3] * backdrop).mul_add(1.0 / 255.0, s[3]),
                ],
                output,
            );
        }
    });
}

/// `αs·αb·HardLight(Cb, Cs)` in units of 255².
#[inline(always)]
fn mixed_hard_light<S: Simd>(
    cs: u16x16<S>,
    sa: u16x16<S>,
    cb: u16x16<S>,
    ba: u16x16<S>,
) -> u16x16<S> {
    let multiply = (cs * cb) << 1;
    let screen = sa * ba - (((ba - cb) * (sa - cs)) << 1);

    (cs << 1).simd_le(sa).select(multiply, screen)
}

#[inline(always)]
fn scaled<S: Simd>(s: [f32x8<S>; 4], b: [f32x8<S>; 4]) -> ([f32x8<S>; 3], [f32x8<S>; 3], f32x8<S>) {
    // The non-separable functions scale with their input when the range of colors scales with it,
    // so they can run on `Cs` and `Cb` scaled by `αs·αb`, which needs no division by the alphas.
    (
        [s[0] * b[3], s[1] * b[3], s[2] * b[3]],
        [b[0] * s[3], b[1] * s[3], b[2] * s[3]],
        s[3] * b[3],
    )
}

#[inline(always)]
fn dodge<S: Simd>(cs: f32x8<S>, sa: f32x8<S>, cb: f32x8<S>, ba: f32x8<S>) -> f32x8<S> {
    let both = sa * ba;
    let mixed = cs
        .simd_ge(sa)
        .select(both, (cb * sa * sa / (sa - cs)).min(both));

    cb.simd_le(0.0).select(f32x8::splat(cs.simd, 0.0), mixed)
}

#[inline(always)]
fn burn<S: Simd>(cs: f32x8<S>, sa: f32x8<S>, cb: f32x8<S>, ba: f32x8<S>) -> f32x8<S> {
    let both = sa * ba;
    let mixed = both - ((ba - cb) * sa * sa / cs).min(both);
    let mixed = cs.simd_le(0.0).select(f32x8::splat(cs.simd, 0.0), mixed);

    cb.simd_ge(ba).select(both, mixed)
}

#[inline(always)]
fn soft<S: Simd>(s: f32x8<S>, b: f32x8<S>) -> f32x8<S> {
    let d = b
        .simd_le(0.25)
        .select((b.mul_add(16.0, -12.0)).mul_add(b, 4.0) * b, b.sqrt());
    let darken = b - (1.0 - 2.0 * s) * b * (1.0 - b);
    let lighten = (2.0 * s - 1.0).mul_add(d - b, b);

    s.simd_le(0.5).select(darken, lighten)
}

#[inline(always)]
fn lum<S: Simd>([r, g, b]: [f32x8<S>; 3]) -> f32x8<S> {
    r.mul_add(0.3, g.mul_add(0.59, b * 0.11))
}

#[inline(always)]
fn sat<S: Simd>([r, g, b]: [f32x8<S>; 3]) -> f32x8<S> {
    r.max(g).max(b) - r.min(g).min(b)
}

#[inline(always)]
fn set_lum<S: Simd>(color: [f32x8<S>; 3], l: f32x8<S>, bound: f32x8<S>) -> [f32x8<S>; 3] {
    let d = l - lum(color);
    let [r, g, b] = [color[0] + d, color[1] + d, color[2] + d];
    let (n, x) = (r.min(g).min(b), r.max(g).max(b));
    // Both steps of `ClipColor` scale the color around `l` by a factor that is the same for every
    // component, so they combine into one scale per pixel.
    let one = f32x8::splat(l.simd, 1.0);
    let scale = n.simd_lt(0.0).select(l / (l - n), one)
        * x.simd_gt(bound).select((bound - l) / (x - l), one);

    [
        (r - l).mul_add(scale, l),
        (g - l).mul_add(scale, l),
        (b - l).mul_add(scale, l),
    ]
}

#[inline(always)]
fn set_sat<S: Simd>(color: [f32x8<S>; 3], s: f32x8<S>) -> [f32x8<S>; 3] {
    let [r, g, b] = color;
    let (n, x) = (r.min(g).min(b), r.max(g).max(b));
    // Moving the minimum to 0 and scaling by `s / (max − min)` sets the maximum to `s` and the
    // middle component as `SetSat` does, without sorting the components.
    let scale = s / (x - n);
    let saturated = x.simd_gt(n);
    let zero = f32x8::splat(s.simd, 0.0);

    [
        saturated.select((r - n) * scale, zero),
        saturated.select((g - n) * scale, zero),
        saturated.select((b - n) * scale, zero),
    ]
}
