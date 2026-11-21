import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    first: Rect,
    second: Rect,
    third: Rect,
    fourth: Rect,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var first: texture_2d<f32>;
@group(0) @binding(2) var second: texture_2d<f32>;
@group(0) @binding(3) var third: texture_2d<f32>;
@group(0) @binding(4) var fourth: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let bottom = over(load(second, params.second, p), load(first, params.first, p));
    return over(load(fourth, params.fourth, p), over(load(third, params.third, p), bottom));
}

fn over(source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    return source + backdrop * (1.0 - source.a);
}
