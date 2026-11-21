import package::util::image::{Rect, load_along, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    axis: vec4<i32>,
    // The weights of the distances 0 to 6.
    weights: array<vec4<f32>, 2>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let at = dot(p, params.axis.xy);
    let w = params.weights;
    var sum = w[0].x * tap(p, at);
    sum += w[0].y * (tap(p, at - 1) + tap(p, at + 1));
    sum += w[0].z * (tap(p, at - 2) + tap(p, at + 2));
    sum += w[0].w * (tap(p, at - 3) + tap(p, at + 3));
    sum += w[1].x * (tap(p, at - 4) + tap(p, at + 4));
    sum += w[1].y * (tap(p, at - 5) + tap(p, at + 5));
    sum += w[1].z * (tap(p, at - 6) + tap(p, at + 6));
    return sum;
}

fn tap(p: vec2<i32>, at: i32) -> vec4<f32> {
    return load_along(input, params.input, p, params.axis.xy, at);
}
