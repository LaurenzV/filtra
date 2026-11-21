import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    cell: vec4<i32>,
    // A start of a repetition at or before the output, so that `%` sees no negative operands.
    start: vec4<i32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let size = params.cell.zw - params.cell.xy;
    let p = (pixel(position, params.output) - params.start.xy) % size;
    return load(input, params.input, params.cell.xy + p);
}
