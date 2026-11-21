import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    source: Rect,
    backdrop: Rect,
    // Over, in, out, atop, xor, lighter or arithmetic.
    mode: vec4<u32>,
    k: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var backdrop: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let s = load(source, params.source, p);
    let b = load(backdrop, params.backdrop, p);

    switch params.mode.x {
        case 0u: {
            return s + b * (1.0 - s.a);
        }
        case 1u: {
            return s * b.a;
        }
        case 2u: {
            return s * (1.0 - b.a);
        }
        case 3u: {
            return s * b.a + b * (1.0 - s.a);
        }
        case 4u: {
            return s * (1.0 - b.a) + b * (1.0 - s.a);
        }
        case 5u: {
            return min(s + b, vec4<f32>(1.0));
        }
        default: {
            let k = params.k;
            let result = k.x * s * b + k.y * s + k.z * b + k.w;
            let alpha = clamp(result.a, 0.0, 1.0);
            return vec4<f32>(clamp(result.rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
        }
    }
}
