import package::util::image::{Rect, load, pixel};

struct Params {
    output: vec4<i32>,
    input: Rect,
    // The pixels lit, whose edges use the one-sided kernels.
    bounds: vec4<i32>,
    // The surface scale and the pixels per user unit along x and y.
    surface: vec4<f32>,
    // The position of the user-space origin in pixels.
    origin: vec4<f32>,
    color: vec4<f32>,
    // Distant, point or spot light, whether specular and whether the spot has a cone.
    kinds: vec4<u32>,
    // The reflection constant and exponent, the spot exponent and the cosine of the cone.
    constants: vec4<f32>,
    // The direction to a distant light or the position of the others.
    position: vec4<f32>,
    // The direction of a spot light.
    direction: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input: texture_2d<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pixel(position, params.output);
    let slope = gradient(p);
    let surface = params.surface;
    let n = vec3<f32>(-surface.x * surface.yz * slope, 1.0);
    let normal = n / length(n);
    let user = (vec2<f32>(p) - params.origin.xy) / surface.yz;
    let point = vec3<f32>(user, alpha(p) * surface.x);

    var to_light = params.position.xyz;
    var intensity = 1.0;

    if params.kinds.x != 0u {
        let difference = params.position.xyz - point;
        let inverse_distance = 1.0 / length(difference);
        to_light = difference * inverse_distance;

        if params.kinds.x == 2u {
            let axis = params.direction.xyz;
            let cosine = -dot(to_light, axis);
            intensity = select(0.0, power(cosine, params.constants.z), cosine > 0.0);

            // Anti-aliases the edge of the cone over one pixel, with the change of the cosine
            // across a pixel from its derivatives along x and y, which are
            // `(S + cosine · L) / distance`.
            if params.kinds.z != 0u {
                let change = (cosine * to_light.xy + axis.xy) * inverse_distance / surface.yz;
                let coverage = (cosine - params.constants.w) / length(change) + 0.5;
                intensity *= clamp(coverage, 0.0, 1.0);
            }
        }
    }

    if params.kinds.y == 0u {
        let factor = dot(normal, to_light) * intensity * params.constants.x;
        return vec4<f32>(clamp(factor * params.color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
    }

    // The halfway vector between the light and the eye, which is at (0, 0, 1).
    let halfway = to_light + vec3<f32>(0.0, 0.0, 1.0);
    let cosine = dot(normal, halfway) / length(halfway);
    let factor = power(max(cosine, 0.0), params.constants.y) * intensity * params.constants.x;
    let color = clamp(factor * params.color.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(color, max(max(color.r, color.g), color.b));
}

// The Sobel kernels of §9.10, with their one-sided forms at the edges of the lit pixels.
fn gradient(p: vec2<i32>) -> vec2<f32> {
    let bounds = params.bounds;
    let above = f32(p.y > bounds.y);
    let below = f32(p.y + 1 < bounds.w);
    let up = p - vec2<i32>(0, i32(above));
    let down = p + vec2<i32>(0, i32(below));
    let left = vec2<i32>(1, 0);

    let horizontal = above * (extended_alpha(up + left) - extended_alpha(up - left))
        + 2.0 * (extended_alpha(p + left) - extended_alpha(p - left))
        + below * (extended_alpha(down + left) - extended_alpha(down - left));
    let x = horizontal / (above + 2.0 + below);

    // A difference over one row instead of two counts twice.
    let span = above + below;
    let scale = select(0.0, 2.0 / span, span > 0.0);
    let center = column(p, up, down, scale);
    var y = 0.0;

    if bounds.z - bounds.x == 1 {
        y = center;
    } else if p.x == bounds.x {
        y = (2.0 * center + column(p + left, up + left, down + left, scale)) / 3.0;
    } else if p.x == bounds.z - 1 {
        y = (2.0 * center + column(p - left, up - left, down - left, scale)) / 3.0;
    } else {
        let sides = column(p - left, up - left, down - left, scale)
            + column(p + left, up + left, down + left, scale);
        y = (sides + 2.0 * center) / 4.0;
    }

    return vec2<f32>(x, y);
}

fn column(p: vec2<i32>, up: vec2<i32>, down: vec2<i32>, scale: f32) -> f32 {
    return scale * (alpha(down) - alpha(up));
}

fn extended_alpha(p: vec2<i32>) -> f32 {
    let bounds = params.bounds;
    let left = vec2<i32>(1, 0);
    let single = bounds.z - bounds.x == 1;

    if p.x < bounds.x {
        let edge = p + left;
        return select(2.0 * alpha(edge) - alpha(edge + left), alpha(edge), single);
    }

    if p.x >= bounds.z {
        let edge = p - left;
        return select(2.0 * alpha(edge) - alpha(edge - left), alpha(edge), single);
    }

    return alpha(p);
}

fn alpha(p: vec2<i32>) -> f32 {
    return load(input, params.input, p).a;
}

fn power(base: f32, exponent: f32) -> f32 {
    // `0⁰` is 1, while `pow` is undefined for a base of 0.
    if exponent == 0.0 {
        return 1.0;
    }

    return select(pow(base, exponent), 0.0, base <= 0.0);
}
