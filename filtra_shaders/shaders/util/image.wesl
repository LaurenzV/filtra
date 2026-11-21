struct Rect {
    origin: vec4<i32>,
    bounds: vec4<i32>,
}

fn pixel(position: vec4<f32>, origin: vec4<i32>) -> vec2<i32> {
    return vec2<i32>(floor(position.xy)) + origin.xy;
}

fn load(texture: texture_2d<f32>, rect: Rect, pixel: vec2<i32>) -> vec4<f32> {
    if any(pixel < rect.bounds.xy) || any(pixel >= rect.bounds.zw) {
        return vec4<f32>(0.0);
    }

    return textureLoad(texture, pixel - rect.origin.xy, 0);
}

fn load_along(texture: texture_2d<f32>, rect: Rect, pixel: vec2<i32>, axis: vec2<i32>, at: i32) -> vec4<f32> {
    return load(texture, rect, pixel + (at - dot(pixel, axis)) * axis);
}
