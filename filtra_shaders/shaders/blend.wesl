import package::util::color::unpremultiply;
import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    source: Rect,
    backdrop: Rect,
    // The blend mode in the order of `BlendMode`, and whether to composite onto the backdrop.
    mode: vec4<u32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var backdrop: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let s = load(source, params.source, p);
    let b = load(backdrop, params.backdrop, p);
    let both = s.a * b.a;
    let composite = f32(params.mode.y);
    // `αs·αb·B(Cb, Cs)`, which works on premultiplied colors.
    let mixed = clamp(mix(params.mode.x, s, b), vec3<f32>(0.0), vec3<f32>(both));
    let color = s.rgb * (1.0 - b.a) + b.rgb * (1.0 - s.a) * composite + mixed;
    return vec4<f32>(color, s.a + b.a * (1.0 - s.a) * composite);
}

fn mix(mode: u32, s: vec4<f32>, b: vec4<f32>) -> vec3<f32> {
    let cs = s.rgb;
    let cb = b.rgb;
    let sa = s.a;
    let ba = b.a;

    switch mode {
        case 1u: {
            return cs * cb;
        }
        case 2u: {
            return cs * ba + cb * sa - cs * cb;
        }
        case 3u: {
            return hard_light(cb, ba, cs, sa);
        }
        case 4u: {
            return min(cs * ba, cb * sa);
        }
        case 5u: {
            return max(cs * ba, cb * sa);
        }
        case 6u: {
            let dodge = select(min(cb * sa * sa / (sa - cs), vec3<f32>(sa * ba)), vec3<f32>(sa * ba), cs >= vec3<f32>(sa));
            return select(dodge, vec3<f32>(0.0), cb <= vec3<f32>(0.0));
        }
        case 7u: {
            let burn = sa * ba - min((ba - cb) * sa * sa / cs, vec3<f32>(sa * ba));
            return select(select(burn, vec3<f32>(0.0), cs <= vec3<f32>(0.0)), vec3<f32>(sa * ba), cb >= vec3<f32>(ba));
        }
        case 8u: {
            return hard_light(cs, sa, cb, ba);
        }
        case 9u: {
            let s = unpremultiply(s).rgb;
            let b = unpremultiply(b).rgb;
            let d = select(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, b <= vec3<f32>(0.25));
            let darken = b - (1.0 - 2.0 * s) * b * (1.0 - b);
            let lighten = b + (2.0 * s - 1.0) * (d - b);
            return select(lighten, darken, s <= vec3<f32>(0.5)) * sa * ba;
        }
        case 10u: {
            return abs(cs * ba - cb * sa);
        }
        case 11u: {
            return cs * ba + cb * sa - 2.0 * cs * cb;
        }
        // The non-separable modes scale with the colors, so they run on `Cs` and `Cb` scaled by
        // `αs·αb`, which needs no division by the alphas.
        case 12u: {
            return set_lum(set_sat(cs * ba, sat(cb * sa)), lum(cb * sa), sa * ba);
        }
        case 13u: {
            return set_lum(set_sat(cb * sa, sat(cs * ba)), lum(cb * sa), sa * ba);
        }
        case 14u: {
            return set_lum(cs * ba, lum(cb * sa), sa * ba);
        }
        case 15u: {
            return set_lum(cb * sa, lum(cs * ba), sa * ba);
        }
        default: {
            return cs * ba;
        }
    }
}

fn hard_light(cs: vec3<f32>, sa: f32, cb: vec3<f32>, ba: f32) -> vec3<f32> {
    let multiply = 2.0 * cs * cb;
    let screen = sa * ba - 2.0 * (ba - cb) * (sa - cs);
    return select(screen, multiply, 2.0 * cs <= vec3<f32>(sa));
}

fn lum(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.3, 0.59, 0.11));
}

fn sat(color: vec3<f32>) -> f32 {
    return max(max(color.r, color.g), color.b) - min(min(color.r, color.g), color.b);
}

fn set_lum(color: vec3<f32>, l: f32, bound: f32) -> vec3<f32> {
    let c = color + (l - lum(color));
    let n = min(min(c.r, c.g), c.b);
    let x = max(max(c.r, c.g), c.b);
    // Both steps of `ClipColor` scale the color around `l` by the same factor for every component.
    let scale = select(1.0, l / (l - n), n < 0.0) * select(1.0, (bound - l) / (x - l), x > bound);
    return (c - l) * scale + l;
}

fn set_sat(color: vec3<f32>, s: f32) -> vec3<f32> {
    let n = min(min(color.r, color.g), color.b);
    let x = max(max(color.r, color.g), color.b);
    // Moving the minimum to 0 and scaling by `s / (max − min)` sets the maximum to `s` and the
    // middle component as `SetSat` does, without sorting the components.
    return select(vec3<f32>(0.0), (color - n) * (s / (x - n)), x > n);
}
