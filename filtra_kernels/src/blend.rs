use fearless_simd::{Select, Simd, f32x8, prelude::*, u16x16, u32x8};
use filtra_types::BlendMode;

use crate::dispatch::dispatch;
use crate::simd::{Wide, div255, for_each_pixel_chunk, pack, unpack, unpremultiply};
use crate::{Source, Target};

/// `feBlend` and `mix-blend-mode`.
pub fn blend(
    src: Source<'_>,
    backdrop: Source<'_>,
    dst: Target<'_>,
    mode: BlendMode,
    composite: bool,
) {
    let inputs = [src, backdrop];

    match mode {
        BlendMode::ColorDodge
        | BlendMode::ColorBurn
        | BlendMode::SoftLight
        | BlendMode::Hue
        | BlendMode::Saturation
        | BlendMode::Color
        | BlendMode::Luminosity => {
            dispatch!(simd => float(simd, inputs, composite, dst, {
                #[inline(always)]
                |s, b| float_mixed(mode, s, b)
            }))
        }
        _ => dispatch!(simd => integer(simd, inputs, composite, dst, {
            #[inline(always)]
            |cs, sa, cb, ba| integer_mixed(mode, cs, sa, cb, ba)
        })),
    }
}

#[inline(always)]
fn integer_mixed<S: Simd>(
    mode: BlendMode,
    cs: u16x16<S>,
    sa: u16x16<S>,
    cb: u16x16<S>,
    ba: u16x16<S>,
) -> u16x16<S> {
    match mode {
        BlendMode::Multiply => cs * cb,
        BlendMode::Screen => cs * ba + cb * sa - cs * cb,
        BlendMode::Overlay => mixed_hard_light(cb, ba, cs, sa),
        BlendMode::Darken => (cs * ba).min(cb * sa),
        BlendMode::Lighten => (cs * ba).max(cb * sa),
        BlendMode::HardLight => mixed_hard_light(cs, sa, cb, ba),
        BlendMode::Difference => {
            let (s, b) = (cs * ba, cb * sa);
            s.max(b) - s.min(b)
        }
        BlendMode::Exclusion => cs * ba + cb * sa - ((cs * cb) << 1),
        _ => cs * ba,
    }
}

#[inline(always)]
fn float_mixed<S: Simd>(mode: BlendMode, s: [f32x8<S>; 4], b: [f32x8<S>; 4]) -> [f32x8<S>; 3] {
    match mode {
        BlendMode::ColorDodge => [
            dodge(s[0], s[3], b[0], b[3]),
            dodge(s[1], s[3], b[1], b[3]),
            dodge(s[2], s[3], b[2], b[3]),
        ],
        BlendMode::ColorBurn => [
            burn(s[0], s[3], b[0], b[3]),
            burn(s[1], s[3], b[1], b[3]),
            burn(s[2], s[3], b[2], b[3]),
        ],
        BlendMode::SoftLight => {
            let both = s[3] * b[3];
            let (s, b) = (unpremultiply(s), unpremultiply(b));
            [
                soft(s[0], b[0]) * both,
                soft(s[1], b[1]) * both,
                soft(s[2], b[2]) * both,
            ]
        }
        BlendMode::Hue => {
            let (s, b, bound) = scaled(s, b);
            set_lum(set_sat(s, sat(b)), lum(b), bound)
        }
        BlendMode::Saturation => {
            let (s, b, bound) = scaled(s, b);
            set_lum(set_sat(b, sat(s)), lum(b), bound)
        }
        BlendMode::Color => {
            let (s, b, bound) = scaled(s, b);
            set_lum(s, lum(b), bound)
        }
        _ => {
            let (s, b, bound) = scaled(s, b);
            set_lum(b, lum(s), bound)
        }
    }
}

#[inline(always)]
fn integer<S: Simd>(
    simd: S,
    inputs: [Source<'_>; 2],
    composite: bool,
    dst: Target<'_>,
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

    for_each_pixel_chunk(inputs, dst, {
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
    inputs: [Source<'_>; 2],
    composite: bool,
    dst: Target<'_>,
    mixed: impl Fn([f32x8<S>; 4], [f32x8<S>; 4]) -> [f32x8<S>; 3] + Copy,
) {
    let backdrop = if composite { 1.0 } else { 0.0 };

    for_each_pixel_chunk(inputs, dst, {
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
