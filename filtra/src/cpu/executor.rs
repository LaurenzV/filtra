use filtra_kernels::{Source, Target};

use super::{Backend, PixmapMut};
use crate::IntRect;
use crate::plan::{ImageId, Plan, Step};

impl Backend {
    pub(crate) fn execute(
        &mut self,
        plan: &Plan,
        pixmap: PixmapMut<'_>,
        render_image: &mut dyn FnMut(usize, IntRect, &mut [u8]),
    ) {
        let assignment = crate::plan::assign_buffers(&plan.steps, &plan.rects, plan.output);
        let mut images: Vec<Image> = (plan.rects.iter().zip(&assignment.images))
            .map(|(&rect, &buffer)| Image {
                location: buffer.map_or(Location::Source, Location::Buffer),
                rect,
            })
            .collect();

        let in_place = plan.steps.last().is_some_and(|last| {
            last.writes() == plan.output
                // `render_image` needs the rows of the image to be contiguous.
                && !matches!(last, Step::Image { .. })
                && last.reads().all(|read| assignment.images[read.0].is_some())
        });

        if in_place {
            images[plan.output.0].location = Location::Output;
        }

        let mut store = Store {
            pixmap,
            images,
            buffers: self.take(assignment.count),
        };

        for step in &plan.steps {
            execute_step(step, &mut store, render_image);
        }

        if !in_place {
            let rect = store.images[plan.output.0].rect;
            store.write_pixmap(rect, |inputs, target| {
                filtra_kernels::offset(inputs.read(plan.output), target, [0, 0]);
            });
        }

        self.keep(store.buffers);
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
                    source: Some(self.pixmap.source()),
                };
                let (width, height) = (rect.width(), rect.height());
                f(
                    inputs,
                    Target::new(sized(&mut data, rect), width, height, rect.x0, rect.y0),
                );
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
        let bounds = self.pixmap.target().rect();

        for outside in [
            IntRect::new(bounds.x0, bounds.y0, bounds.x1, rect.y0),
            IntRect::new(bounds.x0, rect.y1, bounds.x1, bounds.y1),
            IntRect::new(bounds.x0, rect.y0, rect.x0, rect.y1),
            IntRect::new(rect.x1, rect.y0, bounds.x1, rect.y1),
        ] {
            filtra_kernels::flood(self.pixmap.target().clip(outside), [0; 4]);
        }

        let inputs = Inputs {
            images: &self.images,
            buffers: &self.buffers,
            source: None,
        };
        f(inputs, self.pixmap.target().clip(rect));
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
            filtra_kernels::convert(inputs.read(*src), target, *from, *to);
        }),
        Step::Flood { dst, color } => store.write(*dst, |_, target| {
            filtra_kernels::flood(target, *color);
        }),
        Step::Image { dst, node } => {
            let (rect, data) = store.buffer(*dst);
            data.fill(0);

            if !rect.is_empty() {
                render_image(*node, rect, data);
            }
        }
        #[cfg(any(
            feature = "offset",
            feature = "morphology",
            feature = "convolve-matrix"
        ))]
        Step::Offset { src, dst, dx, dy } => store.write(*dst, |inputs, target| {
            filtra_kernels::offset(inputs.read(*src), target, [*dx, *dy]);
        }),
        #[cfg(feature = "tile")]
        Step::Tile { src, cell, dst } => store.write(*dst, |inputs, target| {
            filtra_kernels::tile(inputs.read(*src), target, *cell);
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
            filtra_kernels::composite(src, backdrop, target, *operator);
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
            filtra_kernels::blend(src, backdrop, target, *mode, *composite);
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
            filtra_kernels::morphology(inputs.read(*src), target, *operator, [*rx, *ry]);
        }),
        #[cfg(feature = "displacement-map")]
        Step::DisplacementMap {
            src,
            map,
            dst,
            scale,
            channels,
        } => store.write(*dst, |inputs, target| {
            let (src, map) = (inputs.read(*src), inputs.read(*map));
            filtra_kernels::displacement_map(src, map, target, *scale, *channels);
        }),
        #[cfg(feature = "turbulence")]
        Step::Turbulence { dst, turbulence } => store.write(*dst, |_, target| {
            filtra_kernels::turbulence(target, turbulence);
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
            convolution,
        } => store.write(*dst, |inputs, target| {
            filtra_kernels::convolve_matrix(inputs.read(*src), target, convolution);
        }),
        #[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
        Step::Lighting { src, dst, lighting } => store.write(*dst, |inputs, target| {
            filtra_kernels::lighting(inputs.read(*src), target, lighting);
        }),
        #[cfg(feature = "gaussian-blur")]
        Step::GaussianBlur { src, dst, std_dev } => store.write(*dst, |inputs, target| {
            filtra_kernels::gaussian_blur(inputs.read(*src), target, *std_dev);
        }),
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
