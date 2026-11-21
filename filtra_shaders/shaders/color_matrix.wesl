import package::util::color::{premultiply, unpremultiply};
import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    matrix: mat4x4<f32>,
    offset: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let color = unpremultiply(load(input, params.input, pixel(position, params.output)));
    return premultiply(params.matrix * color + params.offset);
}
