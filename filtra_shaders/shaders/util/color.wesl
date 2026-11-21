fn unpremultiply(color: vec4<f32>) -> vec4<f32> {
    if color.a <= 0.0 {
        return vec4<f32>(0.0);
    }

    return vec4<f32>(color.rgb / color.a, color.a);
}

fn premultiply(color: vec4<f32>) -> vec4<f32> {
    let clamped = clamp(color, vec4<f32>(0.0), vec4<f32>(1.0));
    return vec4<f32>(clamped.rgb * clamped.a, clamped.a);
}
