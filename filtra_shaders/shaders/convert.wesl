import package::util::color::{premultiply, unpremultiply};
import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    to_linear: vec4<u32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let color = unpremultiply(load(input, params.input, pixel(position, params.output)));
    var rgb = to_srgb(color.rgb);

    if params.to_linear.x == 1u {
        rgb = to_linear(color.rgb);
    }

    return premultiply(vec4<f32>(rgb, color.a));
}
