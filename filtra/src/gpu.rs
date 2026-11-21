//! Passes for GPU backends.
//!
//! [`passes`] turns a [`Plan`] into draws of the shaders in [`filtra_shaders`]. A backend runs them
//! in order, each with its uniforms, input textures and viewport.

pub use filtra_shaders::{Shader, VERTEX_GLSL, VERTEX_WGSL};

#[cfg(feature = "gaussian-blur")]
use filtra_core::blur::{Blur, Op, Span, spans};
use filtra_plan::{ImageId, Pass as _, Plan, Step};

use crate::IntRect;

/// The draws that apply a filter.
#[derive(Debug, Clone, PartialEq)]
pub struct Passes {
    /// The passes in execution order.
    pub passes: Vec<Pass>,
    /// The size of each intermediate texture.
    pub buffers: Vec<[u32; 2]>,
}

/// One draw of a fragment shader.
#[derive(Debug, Clone, PartialEq)]
pub struct Pass {
    /// The fragment shader.
    pub shader: Shader,
    /// The textures bound at bindings 1 and up.
    pub inputs: Vec<Texture>,
    /// The texture the pass draws into.
    pub output: Texture,
    /// The pixels of `output` the pass draws, which is also the viewport.
    pub viewport: IntRect,
    /// The uniform buffer bound at binding 0, as 16-byte slots of little-endian `u32`s.
    pub uniforms: Vec<[u32; 4]>,
}

/// A texture of premultiplied RGBA8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Texture {
    /// The image the filter is applied to.
    Target,
    /// An intermediate texture.
    Buffer(usize),
}

/// The passes that execute `plan` on a target covering `bounds`.
///
/// Panics if `plan` has a step without a GPU shader yet.
pub fn passes(plan: &Plan, bounds: IntRect) -> Passes {
    let assignment = filtra_plan::assign_buffers(&plan.steps, &plan.rects, plan.output);
    let mut buffers = vec![[0, 0]; assignment.count];
    let images: Vec<Image> = (plan.rects.iter().zip(&assignment.images))
        .map(|(&rect, buffer)| match *buffer {
            Some(buffer) => {
                let size = &mut buffers[buffer];
                *size = [size[0].max(rect.width()), size[1].max(rect.height())];

                Image {
                    texture: Texture::Buffer(buffer),
                    origin: [rect.x0, rect.y0],
                    rect,
                }
            }
            None => Image {
                texture: Texture::Target,
                origin: [bounds.x0, bounds.y0],
                rect,
            },
        })
        .collect();

    let mut passes = Passes {
        passes: Vec::with_capacity(plan.steps.len() + 2),
        buffers,
    };

    for step in &plan.steps {
        passes.lower(step, &images);
    }

    // The result is copied into the target last, so a result that is the source needs a copy first.
    let mut output = images[plan.output.0];

    if output.texture == Texture::Target {
        let copy = passes.buffer(output.rect);
        passes.draw(Shader::Offset, &[output], copy, &[[0; 4]]);
        output = copy;
    }

    let target = Image {
        texture: Texture::Target,
        origin: [bounds.x0, bounds.y0],
        rect: bounds,
    };
    passes.draw(Shader::Offset, &[output], target, &[[0; 4]]);
    passes
}

#[derive(Debug, Clone, Copy)]
struct Image {
    texture: Texture,
    origin: [i32; 2],
    rect: IntRect,
}

impl Passes {
    fn lower(&mut self, step: &Step, images: &[Image]) {
        let image = |id: ImageId| images[id.0];
        let dst = image(step.writes());

        if dst.rect.is_empty() {
            return;
        }

        match *step {
            Step::Alpha { src, .. } => self.draw(Shader::Alpha, &[image(src)], dst, &[]),
            Step::Convert { src, from, to, .. } if from == to => {
                self.draw(Shader::Offset, &[image(src)], dst, &[[0; 4]])
            }
            Step::Convert { src, to, .. } => {
                let to_linear = u32::from(to == filtra_plan::ColorSpace::LinearRgb);
                self.draw(Shader::Convert, &[image(src)], dst, &[[to_linear, 0, 0, 0]])
            }
            Step::Flood { color, .. } => {
                let color = color.map(|c| (f32::from(c) / 255.0).to_bits());
                self.draw(Shader::Flood, &[], dst, &[color])
            }
            Step::Offset { src, dx, dy, .. } => {
                let offset = [clamp(dx) as u32, clamp(dy) as u32, 0, 0];
                self.draw(Shader::Offset, &[image(src)], dst, &[offset])
            }
            #[cfg(feature = "color-matrix")]
            Step::ColorMatrix {
                src, ref matrix, ..
            } => {
                let column = |j: usize| [0, 1, 2, 3].map(|i| matrix[i * 5 + j].to_bits());
                let params = [column(0), column(1), column(2), column(3), column(4)];
                self.draw(Shader::ColorMatrix, &[image(src)], dst, &params)
            }
            #[cfg(feature = "composite")]
            Step::Composite {
                src,
                backdrop,
                operator,
                ..
            } => {
                use filtra_plan::CompositeOperator;

                let (mode, k) = match operator {
                    CompositeOperator::Over => (0, [0.0; 4]),
                    CompositeOperator::In => (1, [0.0; 4]),
                    CompositeOperator::Out => (2, [0.0; 4]),
                    CompositeOperator::Atop => (3, [0.0; 4]),
                    CompositeOperator::Xor => (4, [0.0; 4]),
                    CompositeOperator::Lighter => (5, [0.0; 4]),
                    CompositeOperator::Arithmetic { k1, k2, k3, k4 } => (6, [k1, k2, k3, k4]),
                };
                let params = [[mode, 0, 0, 0], k.map(f32::to_bits)];
                self.draw(
                    Shader::Composite,
                    &[image(src), image(backdrop)],
                    dst,
                    &params,
                )
            }
            #[cfg(feature = "tile")]
            Step::Tile { src, cell, .. } => {
                if cell.is_empty() {
                    self.draw(Shader::Flood, &[], dst, &[[0; 4]])
                } else {
                    let start = |output: i32, cell: i32, size: u32| {
                        let output = i64::from(output);
                        let phase = (output - i64::from(cell)).rem_euclid(i64::from(size));
                        clamp(output - phase) as u32
                    };
                    let start = [
                        start(dst.rect.x0, cell.x0, cell.width()),
                        start(dst.rect.y0, cell.y0, cell.height()),
                        0,
                        0,
                    ];
                    let cell = [cell.x0, cell.y0, cell.x1, cell.y1].map(|v| v as u32);
                    self.draw(Shader::Tile, &[image(src)], dst, &[cell, start])
                }
            }
            #[cfg(feature = "blend")]
            Step::Blend {
                src,
                backdrop,
                mode,
                composite,
                ..
            } => {
                let params = [[mode as u32, u32::from(composite), 0, 0]];
                self.draw(Shader::Blend, &[image(src), image(backdrop)], dst, &params)
            }
            #[cfg(feature = "displacement-map")]
            Step::DisplacementMap {
                src,
                map,
                scale,
                channels,
                ..
            } => {
                // An infinite scale would make a channel of one half displace by NaN.
                let [x, y] = scale.map(|scale| scale.clamp(-f32::MAX, f32::MAX).to_bits());
                let [cx, cy] = channels.map(|channel| channel as u32);
                let params = [[x, y, 0, 0], [cx, cy, 0, 0]];
                self.draw(
                    Shader::DisplacementMap,
                    &[image(src), image(map)],
                    dst,
                    &params,
                )
            }
            #[cfg(feature = "morphology")]
            Step::Morphology {
                src,
                operator,
                rx,
                ry,
                ..
            } => {
                let src = image(src);
                let dilate = operator == filtra_plan::MorphologyOperator::Dilate;
                let x0 = (i64::from(dst.rect.x0) - i64::from(rx)).max(i64::from(src.rect.x0));
                let x1 = (i64::from(dst.rect.x1) + i64::from(rx)).min(i64::from(src.rect.x1));

                if x0 >= x1 {
                    self.draw(Shader::Flood, &[], dst, &[[0; 4]])
                } else {
                    let rect = IntRect::new(x0 as i32, dst.rect.y0, x1 as i32, dst.rect.y1);
                    let columns = self.buffer(rect);
                    self.morphology(src, columns, Axis::Y, ry, dilate);
                    self.morphology(columns, dst, Axis::X, rx, dilate);
                }
            }
            #[cfg(feature = "merge")]
            Step::Merge { ref srcs, .. } => self.merge(srcs.iter().map(|&src| image(src)), dst),
            #[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
            Step::Lighting {
                src,
                reflection,
                surface_scale,
                light,
                color,
                scale,
                origin,
                ..
            } => {
                use filtra_plan::{Light, Reflection};

                let (specular, constant, exponent) = match reflection {
                    Reflection::Diffuse { constant } => (0, constant, 0.0),
                    Reflection::Specular { constant, exponent } => (1, constant, exponent),
                };
                let (kind, position, direction, spot_exponent, cone) = match light {
                    Light::Distant { direction } => (0, direction, [0.0; 3], 0.0, None),
                    Light::Point { position } => (1, position, [0.0; 3], 0.0, None),
                    Light::Spot {
                        position,
                        direction,
                        exponent,
                        cone,
                    } => (2, position, direction, exponent, cone),
                };
                let bits = |values: [f32; 4]| values.map(f32::to_bits);
                let bounds = dst.rect;
                let params = [
                    [bounds.x0, bounds.y0, bounds.x1, bounds.y1].map(|v| v as u32),
                    bits([surface_scale, scale[0], scale[1], 0.0]),
                    bits([origin[0], origin[1], 0.0, 0.0]),
                    bits([color[0], color[1], color[2], 0.0]),
                    [kind, specular, u32::from(cone.is_some()), 0],
                    bits([constant, exponent, spot_exponent, cone.unwrap_or(0.0)]),
                    bits([position[0], position[1], position[2], 0.0]),
                    bits([direction[0], direction[1], direction[2], 0.0]),
                ];
                self.draw(Shader::Lighting, &[image(src)], dst, &params);
            }
            #[cfg(feature = "gaussian-blur")]
            Step::GaussianBlur { src, std_dev, .. } => {
                self.gaussian_blur(image(src), dst, std_dev);
            }
            _ => unimplemented!("GPU passes for {step:?}"),
        }
    }

    // Each pass takes the extreme of 8 windows of the previous pass, overlapping where the window it
    // computes is less than 8 times as wide, so the number of passes grows with the logarithm of
    // the radius. A window at `x` covers `[x, x + width)`, and the last pass moves them by the
    // radius to center them.
    #[cfg(feature = "morphology")]
    fn morphology(&mut self, src: Image, dst: Image, axis: Axis, radius: u32, dilate: bool) {
        const TAPS: i64 = 8;

        let (src0, src1) = axis.range(src.rect);
        let (dst0, dst1) = axis.range(dst.rect);
        // A window spanning both the input and the output covers the input and transparent pixels
        // on both sides, as any larger window does.
        let span = src1.max(dst1) - src0.min(dst0);
        let radius = i64::from(radius).min(span);
        let mut widths = vec![1];

        while widths.len() < 2 || widths[widths.len() - 1] < 2 * radius + 1 {
            widths.push((widths[widths.len() - 1] * TAPS).min(2 * radius + 1));
        }

        let last = widths.len() - 1;
        let shift = |level: usize| if level == last { -radius } else { 0 };
        let mut ranges = vec![(dst0, dst1); widths.len()];

        // Windows outside the input are transparent like the pixels outside an image, so a level
        // only needs the windows that reach into the input.
        for level in (1..last).rev() {
            let (lo, hi) = ranges[level + 1];
            let shift = shift(level + 1);
            let reach = widths[level + 1] - widths[level];
            ranges[level] = (
                (lo + shift).max(src0 - widths[level] + 1),
                (hi + shift + reach).min(src1),
            );

            if ranges[level].0 >= ranges[level].1 {
                return self.draw(Shader::Flood, &[], dst, &[[0; 4]]);
            }
        }

        let mut input = src;

        for level in 1..=last {
            let output = if level == last {
                dst
            } else {
                self.buffer(axis.with_range(dst.rect, ranges[level]))
            };
            let [x, y] = axis.direction();
            let settings = [x, y, shift(level) as u32, u32::from(dilate)];
            let step = widths[level - 1] as u32;
            let reach = (widths[level] - widths[level - 1]) as u32;
            self.draw(
                Shader::Morphology,
                &[input],
                output,
                &[settings, [step, reach, 0, 0]],
            );
            input = output;
        }
    }

    // Source over is associative, so each pass stacks up to four images onto the result so far.
    #[cfg(feature = "merge")]
    fn merge(&mut self, srcs: impl Iterator<Item = Image>, dst: Image) {
        let mut srcs = srcs.peekable();
        let Some(&first) = srcs.peek() else {
            return self.draw(Shader::Flood, &[], dst, &[[0; 4]]);
        };
        let unused = Image {
            rect: IntRect::new(0, 0, 0, 0),
            ..first
        };
        let mut spares = [None, None];
        let mut inputs = Vec::with_capacity(4);

        loop {
            inputs.extend(srcs.by_ref().take(4 - inputs.len()));
            inputs.resize(4, unused);

            if srcs.peek().is_none() {
                return self.draw(Shader::Merge, &inputs, dst, &[]);
            }

            // Two buffers take turns, as a pass can't write the texture it reads.
            spares.swap(0, 1);
            let output = *spares[0].get_or_insert_with(|| self.buffer(dst.rect));
            self.draw(Shader::Merge, &inputs, output, &[]);
            inputs.clear();
            inputs.push(output);
        }
    }

    // Each pass becomes one draw per axis that it changes.
    #[cfg(feature = "gaussian-blur")]
    fn gaussian_blur(&mut self, src: Image, dst: Image, std_dev: [f32; 2]) {
        let (passes, weights) = match Blur::new(std_dev, src.rect, dst.rect) {
            Blur::Unchanged => return self.draw(Shader::Offset, &[src], dst, &[[0; 4]]),
            Blur::Transparent => return self.draw(Shader::Flood, &[], dst, &[[0; 4]]),
            Blur::Decimated { passes, weights } => (passes, weights),
        };
        let mut input = src;

        for (index, pass) in passes.iter().enumerate() {
            let [x, y] = pass.ops;
            let last = index + 1 == passes.len();
            let pass_output = |this: &mut Self| {
                if last {
                    Some(dst)
                } else {
                    rect(pass.spans[0], pass.spans[1]).map(|rect| this.buffer(rect))
                }
            };

            if x != Op::Copy {
                let output = if y == Op::Copy {
                    pass_output(self)
                } else {
                    // The vertical draw that follows only reads some rows of this one.
                    let rows = reads(y, pass.spans[1], &weights[1]);
                    let rows = rows.intersect(spans(input.rect)[1]);
                    rect(pass.spans[0], rows).map(|rect| self.buffer(rect))
                };

                let Some(output) = output else {
                    return self.draw(Shader::Flood, &[], dst, &[[0; 4]]);
                };

                self.blur_axis(Axis::X, x, &weights[0], input, output);
                input = output;
            }

            if y != Op::Copy {
                let Some(output) = pass_output(self) else {
                    return self.draw(Shader::Flood, &[], dst, &[[0; 4]]);
                };

                self.blur_axis(Axis::Y, y, &weights[1], input, output);
                input = output;
            }
        }
    }

    #[cfg(feature = "gaussian-blur")]
    fn blur_axis(&mut self, axis: Axis, op: Op, weights: &[u16], input: Image, output: Image) {
        let [x, y] = axis.direction();
        let axis = [x, y, 0, 0];

        match op {
            Op::Halve => self.draw(Shader::Downsample, &[input], output, &[axis]),
            Op::Double => self.draw(Shader::Upsample, &[input], output, &[axis]),
            Op::Gauss => {
                let mut w = [0; 8];

                for (w, &fixed) in w.iter_mut().zip(weights) {
                    *w = (f32::from(fixed) / 32768.0).to_bits();
                }

                let weights = [[w[0], w[1], w[2], w[3]], [w[4], w[5], w[6], w[7]]];
                self.draw(
                    Shader::GaussianBlur,
                    &[input],
                    output,
                    &[axis, weights[0], weights[1]],
                );
            }
            Op::Copy => unreachable!(),
        }
    }

    fn buffer(&mut self, rect: IntRect) -> Image {
        self.buffers.push([rect.width(), rect.height()]);

        Image {
            texture: Texture::Buffer(self.buffers.len() - 1),
            origin: [rect.x0, rect.y0],
            rect,
        }
    }

    // The uniforms are the origin of the viewport, the placement of each input and then `params`.
    fn draw(&mut self, shader: Shader, inputs: &[Image], dst: Image, params: &[[u32; 4]]) {
        if dst.rect.is_empty() {
            return;
        }

        let viewport = IntRect::new(
            dst.rect.x0 - dst.origin[0],
            dst.rect.y0 - dst.origin[1],
            dst.rect.x1 - dst.origin[0],
            dst.rect.y1 - dst.origin[1],
        );
        let mut uniforms = vec![[dst.origin[0] as u32, dst.origin[1] as u32, 0, 0]];

        for input in inputs {
            let rect = input.rect;
            uniforms.push([input.origin[0] as u32, input.origin[1] as u32, 0, 0]);
            uniforms.push([rect.x0, rect.y0, rect.x1, rect.y1].map(|v| v as u32));
        }

        uniforms.extend_from_slice(params);

        self.passes.push(Pass {
            shader,
            inputs: inputs.iter().map(|input| input.texture).collect(),
            output: dst.texture,
            viewport,
            uniforms,
        });
    }
}

#[cfg(any(feature = "morphology", feature = "gaussian-blur"))]
#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
}

#[cfg(any(feature = "morphology", feature = "gaussian-blur"))]
impl Axis {
    #[cfg(feature = "morphology")]
    fn range(self, rect: IntRect) -> (i64, i64) {
        match self {
            Self::X => (i64::from(rect.x0), i64::from(rect.x1)),
            Self::Y => (i64::from(rect.y0), i64::from(rect.y1)),
        }
    }

    #[cfg(feature = "morphology")]
    fn with_range(self, rect: IntRect, (start, end): (i64, i64)) -> IntRect {
        let (start, end) = (clamp(start), clamp(end));

        match self {
            Self::X => IntRect::new(start, rect.y0, end, rect.y1),
            Self::Y => IntRect::new(rect.x0, start, rect.x1, end),
        }
    }

    fn direction(self) -> [u32; 2] {
        match self {
            Self::X => [1, 0],
            Self::Y => [0, 1],
        }
    }
}

#[cfg(feature = "gaussian-blur")]
fn reads(op: Op, span: Span, weights: &[u16]) -> Span {
    match op {
        Op::Copy => span,
        Op::Halve => span.doubled(),
        Op::Double => span.doubling_reads(),
        Op::Gauss => span.grow(weights.len() as i64 - 1),
    }
}

#[cfg(feature = "gaussian-blur")]
fn rect(x: Span, y: Span) -> Option<IntRect> {
    let rect = IntRect::new(clamp(x.0), clamp(y.0), clamp(x.1), clamp(y.1));
    (!rect.is_empty()).then_some(rect)
}

fn clamp(offset: i64) -> i32 {
    offset.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
