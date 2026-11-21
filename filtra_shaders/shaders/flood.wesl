struct Params {
    output: vec4<i32>,
    color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: Params;

@fragment
fn main() -> @location(0) vec4<f32> {
    return params.color;
}
