import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    // The axis, the offset of the first tap and whether to dilate.
    settings: vec4<i32>,
    // The distance between taps and the offset of the last one.
    taps: vec4<i32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let a = extreme(extreme(tap(p, 0), tap(p, 1)), extreme(tap(p, 2), tap(p, 3)));
    let b = extreme(extreme(tap(p, 4), tap(p, 5)), extreme(tap(p, 6), tap(p, 7)));
    return extreme(a, b);
}

fn tap(p: vec2<i32>, index: i32) -> vec4<f32> {
    let offset = params.settings.z + min(index * params.taps.x, params.taps.y);
    return load(input, params.input, p + offset * params.settings.xy);
}

fn extreme(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return select(min(a, b), max(a, b), params.settings.w != 0);
}
