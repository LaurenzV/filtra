import package::util::image::{Rect, load_along, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    axis: vec4<i32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

// [1, 3] / 4 or [3, 1] / 4 between the input pixels around `at / 2`.
@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let axis = params.axis.xy;
    let at = dot(p, axis);
    let start = (at - 1) >> 1u;
    let near = select(0.25, 0.75, (at & 1) != 0);
    let first = load_along(input, params.input, p, axis, start);
    let second = load_along(input, params.input, p, axis, start + 1);
    return near * first + (1.0 - near) * second;
}
