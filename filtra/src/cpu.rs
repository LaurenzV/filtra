#[cfg(feature = "blend")]
use filtra_kernels::BlendMode;
#[cfg(feature = "displacement-map")]
use filtra_kernels::ColorChannel;
#[cfg(feature = "morphology")]
use filtra_kernels::MorphologyOperator;
#[cfg(feature = "composite")]
use filtra_kernels::PorterDuff;
use filtra_kernels::{ColorSpace, IntRect, PixmapMut, Source, Target};
#[cfg(feature = "convolve-matrix")]
use filtra_kernels::{Convolution, EdgeMode};
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
use filtra_kernels::{Light, Lighting, Reflection};
#[cfg(feature = "composite")]
use filtra_plan::CompositeOperator;
use filtra_plan::{ImageId, Pass, Plan, Step};

pub(crate) fn execute(
    plan: &Plan,
    pixmap: PixmapMut<'_>,
    render_image: &mut dyn FnMut(usize, IntRect, &mut [u8]),
) {
    let assignment = filtra_plan::assign_buffers(&plan.steps, &plan.rects, plan.output);
    let mut images: Vec<Image> = (plan.rects.iter().zip(assignment.images))
        .map(|(&rect, buffer)| Image {
            location: buffer.map_or(Location::Source, Location::Buffer),
            rect: kernel_rect(rect),
        })
        .collect();

    let in_place = plan.steps.last().is_some_and(|last| {
        last.writes() == plan.output
            // `render_image` needs the rows of the image to be contiguous.
            && !matches!(last, Step::Image { .. })
            && last.reads().all(|read| Some(read) != plan.source)
    });

    if in_place {
        images[plan.output.0].location = Location::Output;
    }

    let mut store = Store {
        pixmap,
        images,
        buffers: vec![Vec::new(); assignment.count],
    };

    for step in &plan.steps {
        execute_step(step, &mut store, render_image);
    }

    if !in_place {
        let rect = store.images[plan.output.0].rect;
        store.write_pixmap(rect, |inputs, target| {
            filtra_kernels::offset(inputs.read(plan.output), target, 0, 0);
        });
    }
}

#[derive(Clone, Copy)]
struct Image {
    location: Location,
    rect: IntRect,
}

#[derive(Clone, Copy)]
enum Location {
    Source,
    Buffer(usize),
    Output,
}

struct Store<'a> {
    pixmap: PixmapMut<'a>,
    images: Vec<Image>,
    buffers: Vec<Vec<u8>>,
}

impl Store<'_> {
    fn write(&mut self, id: ImageId, f: impl FnOnce(Inputs<'_>, Target<'_>)) {
        let Image { location, rect } = self.images[id.0];

        match location {
            Location::Buffer(buffer) => {
                let mut data = std::mem::take(&mut self.buffers[buffer]);
                let inputs = Inputs {
                    images: &self.images,
                    buffers: &self.buffers,
                    source: Some(self.pixmap.source(0, 0)),
                };
                let mut pixmap =
                    PixmapMut::from_data(sized(&mut data, rect), rect.width(), rect.height());
                f(inputs, pixmap.target(rect.x0, rect.y0));
                self.buffers[buffer] = data;
            }
            Location::Output => self.write_pixmap(rect, f),
            Location::Source => unreachable!("no step writes the source"),
        }
    }

    fn buffer(&mut self, id: ImageId) -> (IntRect, &mut [u8]) {
        let Image { location, rect } = self.images[id.0];
        let Location::Buffer(buffer) = location else {
            unreachable!("{id:?} isn't held by a buffer");
        };

        (rect, sized(&mut self.buffers[buffer], rect))
    }

    fn write_pixmap(&mut self, rect: IntRect, f: impl FnOnce(Inputs<'_>, Target<'_>)) {
        let bounds = self.pixmap.bounds();

        for outside in [
            IntRect::new(bounds.x0, bounds.y0, bounds.x1, rect.y0),
            IntRect::new(bounds.x0, rect.y1, bounds.x1, bounds.y1),
            IntRect::new(bounds.x0, rect.y0, rect.x0, rect.y1),
            IntRect::new(rect.x1, rect.y0, bounds.x1, rect.y1),
        ] {
            filtra_kernels::flood(self.pixmap.target(0, 0).clip(outside), [0; 4]);
        }

        let inputs = Inputs {
            images: &self.images,
            buffers: &self.buffers,
            source: None,
        };
        f(inputs, self.pixmap.target(0, 0).clip(rect));
    }
}

#[derive(Clone, Copy)]
struct Inputs<'a> {
    images: &'a [Image],
    buffers: &'a [Vec<u8>],
    source: Option<Source<'a>>,
}

impl<'a> Inputs<'a> {
    fn read(self, id: ImageId) -> Source<'a> {
        let Image { location, rect } = self.images[id.0];

        match location {
            Location::Source => self.source.expect("the pixmap holds the source").clip(rect),
            Location::Buffer(buffer) => Source::new(
                &self.buffers[buffer][..len(rect)],
                rect.width(),
                rect.height(),
                rect.x0,
                rect.y0,
            ),
            Location::Output => unreachable!("no step reads the output"),
        }
    }
}

fn execute_step(
    step: &Step,
    store: &mut Store<'_>,
    render_image: &mut dyn FnMut(usize, IntRect, &mut [u8]),
) {
    match step {
        Step::Alpha { src, dst } => store.write(*dst, |inputs, target| {
            filtra_kernels::alpha(inputs.read(*src), target);
        }),
        Step::Convert { src, dst, from, to } => store.write(*dst, |inputs, target| {
            let (from, to) = (color_space(*from), color_space(*to));
            filtra_kernels::convert(inputs.read(*src), target, from, to);
        }),
        Step::Flood { dst, color } => store.write(*dst, |_, target| {
            filtra_kernels::flood(target, *color);
        }),
        Step::Image { dst, image } => {
            let (rect, data) = store.buffer(*dst);
            data.fill(0);

            if !rect.is_empty() {
                render_image(*image, rect, data);
            }
        }
        Step::Offset { src, dst, dx, dy } => store.write(*dst, |inputs, target| {
            filtra_kernels::offset(inputs.read(*src), target, *dx, *dy);
        }),
        #[cfg(feature = "tile")]
        Step::Tile { src, cell, dst } => store.write(*dst, |inputs, target| {
            let cell = kernel_rect(*cell);
            filtra_kernels::tile(inputs.read(*src), cell, target);
        }),
        #[cfg(feature = "composite")]
        Step::Composite {
            src,
            backdrop,
            operator,
            dst,
        } => store.write(*dst, |inputs, target| {
            let src = inputs.read(*src);
            let backdrop = inputs.read(*backdrop);
            composite(src, backdrop, *operator, target);
        }),
        #[cfg(feature = "blend")]
        Step::Blend {
            src,
            backdrop,
            mode,
            composite,
            dst,
        } => store.write(*dst, |inputs, target| {
            let src = inputs.read(*src);
            let backdrop = inputs.read(*backdrop);
            filtra_kernels::blend(src, backdrop, blend_mode(*mode), *composite, target);
        }),
        #[cfg(feature = "merge")]
        Step::Merge { srcs, dst } => store.write(*dst, |inputs, target| {
            let srcs: Vec<Source<'_>> = srcs.iter().map(|src| inputs.read(*src)).collect();
            filtra_kernels::merge(&srcs, target);
        }),
        #[cfg(feature = "morphology")]
        Step::Morphology {
            src,
            dst,
            operator,
            rx,
            ry,
        } => store.write(*dst, |inputs, target| {
            let operator = match operator {
                filtra_plan::MorphologyOperator::Erode => MorphologyOperator::Erode,
                filtra_plan::MorphologyOperator::Dilate => MorphologyOperator::Dilate,
            };
            filtra_kernels::morphology(inputs.read(*src), target, operator, *rx, *ry);
        }),
        #[cfg(feature = "displacement-map")]
        Step::DisplacementMap {
            src,
            map,
            dst,
            scale,
            channels,
        } => store.write(*dst, |inputs, target| {
            let channels = channels.map(|channel| match channel {
                filtra_plan::ColorChannel::R => ColorChannel::R,
                filtra_plan::ColorChannel::G => ColorChannel::G,
                filtra_plan::ColorChannel::B => ColorChannel::B,
                filtra_plan::ColorChannel::A => ColorChannel::A,
            });
            let (src, map) = (inputs.read(*src), inputs.read(*map));
            filtra_kernels::displacement_map(src, map, target, *scale, channels);
        }),
        #[cfg(feature = "turbulence")]
        Step::Turbulence {
            dst,
            base_frequency,
            octaves,
            seed,
            stitch_tile,
            fractal_noise,
            origin,
        } => store.write(*dst, |_, target| {
            let turbulence = filtra_kernels::Turbulence {
                base_frequency: *base_frequency,
                octaves: *octaves,
                seed: *seed,
                stitch_tile: stitch_tile.map(kernel_rect),
                fractal_noise: *fractal_noise,
                origin: *origin,
            };
            filtra_kernels::turbulence(target, &turbulence);
        }),
        #[cfg(feature = "color-matrix")]
        Step::ColorMatrix { src, dst, matrix } => store.write(*dst, |inputs, target| {
            filtra_kernels::color_matrix(inputs.read(*src), target, matrix);
        }),
        #[cfg(feature = "component-transfer")]
        Step::ComponentTransfer {
            src,
            dst,
            functions,
        } => store.write(*dst, |inputs, target| {
            let functions = functions.each_ref().map(Vec::as_slice);
            filtra_kernels::component_transfer(inputs.read(*src), target, functions);
        }),
        #[cfg(feature = "convolve-matrix")]
        Step::ConvolveMatrix {
            src,
            dst,
            size,
            weights,
            target,
            bias,
            edge_mode,
            preserve_alpha,
        } => store.write(*dst, |inputs, output| {
            let convolution = Convolution {
                size: *size,
                weights,
                target: *target,
                bias: *bias,
                edge_mode: match edge_mode {
                    filtra_plan::EdgeMode::Duplicate => EdgeMode::Duplicate,
                    filtra_plan::EdgeMode::Wrap => EdgeMode::Wrap,
                    filtra_plan::EdgeMode::Mirror => EdgeMode::Mirror,
                    filtra_plan::EdgeMode::None => EdgeMode::None,
                },
                preserve_alpha: *preserve_alpha,
            };
            filtra_kernels::convolve_matrix(inputs.read(*src), output, &convolution);
        }),
        #[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
        Step::Lighting {
            src,
            dst,
            reflection,
            surface_scale,
            light,
            color,
            scale,
            origin,
        } => store.write(*dst, |inputs, output| {
            let lighting = Lighting {
                reflection: match *reflection {
                    filtra_plan::Reflection::Diffuse { constant } => {
                        Reflection::Diffuse { constant }
                    }
                    filtra_plan::Reflection::Specular { constant, exponent } => {
                        Reflection::Specular { constant, exponent }
                    }
                },
                surface_scale: *surface_scale,
                light: match *light {
                    filtra_plan::Light::Distant { direction } => Light::Distant { direction },
                    filtra_plan::Light::Point { position } => Light::Point { position },
                    filtra_plan::Light::Spot {
                        position,
                        direction,
                        exponent,
                        cone,
                    } => Light::Spot {
                        position,
                        direction,
                        exponent,
                        cone,
                    },
                },
                color: *color,
                scale: *scale,
                origin: *origin,
            };
            filtra_kernels::lighting(inputs.read(*src), output, &lighting);
        }),
        #[cfg(feature = "gaussian-blur")]
        Step::GaussianBlur { src, dst, std_dev } => store.write(*dst, |inputs, target| {
            filtra_kernels::gaussian_blur(inputs.read(*src), target, *std_dev);
        }),
        // A dependency can enable a primitive on `filtra_plan` without enabling it here.
        step => panic!("`filtra` is built without the feature for {step:?}"),
    }
}

fn sized(data: &mut Vec<u8>, rect: IntRect) -> &mut [u8] {
    if data.len() < len(rect) {
        *data = vec![0; len(rect)];
    }

    &mut data[..len(rect)]
}

fn len(rect: IntRect) -> usize {
    rect.width() as usize * rect.height() as usize * 4
}

fn kernel_rect(rect: filtra_plan::IntRect) -> IntRect {
    IntRect::new(rect.x0, rect.y0, rect.x1, rect.y1)
}

#[cfg(feature = "blend")]
fn blend_mode(mode: filtra_plan::BlendMode) -> BlendMode {
    match mode {
        filtra_plan::BlendMode::Normal => BlendMode::Normal,
        filtra_plan::BlendMode::Multiply => BlendMode::Multiply,
        filtra_plan::BlendMode::Screen => BlendMode::Screen,
        filtra_plan::BlendMode::Overlay => BlendMode::Overlay,
        filtra_plan::BlendMode::Darken => BlendMode::Darken,
        filtra_plan::BlendMode::Lighten => BlendMode::Lighten,
        filtra_plan::BlendMode::ColorDodge => BlendMode::ColorDodge,
        filtra_plan::BlendMode::ColorBurn => BlendMode::ColorBurn,
        filtra_plan::BlendMode::HardLight => BlendMode::HardLight,
        filtra_plan::BlendMode::SoftLight => BlendMode::SoftLight,
        filtra_plan::BlendMode::Difference => BlendMode::Difference,
        filtra_plan::BlendMode::Exclusion => BlendMode::Exclusion,
        filtra_plan::BlendMode::Hue => BlendMode::Hue,
        filtra_plan::BlendMode::Saturation => BlendMode::Saturation,
        filtra_plan::BlendMode::Color => BlendMode::Color,
        filtra_plan::BlendMode::Luminosity => BlendMode::Luminosity,
    }
}

fn color_space(space: filtra_plan::ColorSpace) -> ColorSpace {
    match space {
        filtra_plan::ColorSpace::Srgb => ColorSpace::Srgb,
        filtra_plan::ColorSpace::LinearRgb => ColorSpace::LinearRgb,
    }
}

#[cfg(feature = "composite")]
fn composite(src: Source<'_>, backdrop: Source<'_>, operator: CompositeOperator, dst: Target<'_>) {
    let operator = match operator {
        CompositeOperator::Over => PorterDuff::Over,
        CompositeOperator::In => PorterDuff::In,
        CompositeOperator::Out => PorterDuff::Out,
        CompositeOperator::Atop => PorterDuff::Atop,
        CompositeOperator::Xor => PorterDuff::Xor,
        CompositeOperator::Lighter => PorterDuff::Lighter,
        CompositeOperator::Arithmetic { k1, k2, k3, k4 } => {
            return filtra_kernels::arithmetic(src, backdrop, [k1, k2, k3, k4], dst);
        }
    };

    filtra_kernels::composite(src, backdrop, operator, dst);
}
