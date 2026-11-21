import package::util::color::unpremultiply;
import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    map: Rect,
    scale: vec4<f32>,
    channels: vec4<u32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;
@group(0) @binding(2) var map: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let channels = unpremultiply(load(map, params.map, p));
    let shift = vec2<f32>(channels[params.channels.x], channels[params.channels.y]) - 0.5;
    let moved = fma(params.scale.xy, shift, vec2<f32>(p) + 0.5);
    return load(input, params.input, vec2<i32>(floor(moved)));
}
