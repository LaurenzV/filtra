import package::util::image::{Rect, load_along, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    axis: vec4<i32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

// [1, 3, 3, 1] / 8, centered between the input pixels `2 at` and `2 at + 1`.
@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let axis = params.axis.xy;
    let start = 2 * dot(p, axis) - 1;
    let outer = load_along(input, params.input, p, axis, start)
        + load_along(input, params.input, p, axis, start + 3);
    let inner = load_along(input, params.input, p, axis, start + 1)
        + load_along(input, params.input, p, axis, start + 2);
    return (outer + 3.0 * inner) / 8.0;
}
