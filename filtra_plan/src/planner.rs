use filtra_types::{
    ColorInterpolation, ColorMatrix, Filter, Input, Node, Primitive, Rect, TransferFunction,
};

use crate::{ColorSpace, ImageId, IntRect, Plan, Step, Transform};

/// Plans `filter` for a source image covering `bounds`.
pub fn plan(filter: &Filter, transform: Transform, bounds: IntRect) -> Plan {
    let mut planner = Planner::new(filter, transform, bounds);
    let output = planner.plan_output();

    Plan {
        steps: planner.steps,
        rects: planner.rects,
        output,
    }
}

struct Planner<'a> {
    nodes: &'a [Node],
    transform: Transform,
    /// The filter region, clipped to the source.
    region: IntRect,
    subregions: Vec<Rect>,
    /// The outputs of the nodes, followed by `SourceGraphic` and `SourceAlpha`.
    outputs: Vec<Option<Output>>,
    steps: Vec<Step>,
    rects: Vec<IntRect>,
}

#[derive(Clone, Copy)]
struct Output {
    /// The result in `space`.
    image: ImageId,
    space: ColorSpace,
    /// The result in the other color space, if a step needs it.
    converted: Option<ImageId>,
    /// The subregion, not clipped to the filter region.
    subregion: IntRect,
}

impl<'a> Planner<'a> {
    fn new(filter: &'a Filter, transform: Transform, bounds: IntRect) -> Self {
        let nodes = filter.nodes();

        Self {
            nodes,
            transform,
            region: transform.covered_pixels(filter.region).intersect(&bounds),
            subregions: filter.subregions(),
            outputs: vec![None; nodes.len() + 2],
            steps: Vec::new(),
            rects: Vec::new(),
        }
    }

    fn plan_output(&mut self) -> ImageId {
        let Some(last) = self.nodes.len().checked_sub(1) else {
            // A filter without primitives makes the element transparent.
            let dst = self.image(IntRect::default());
            self.steps.push(Step::Flood { dst, color: [0; 4] });

            return dst;
        };

        let mut needed = vec![false; self.nodes.len()];
        needed[last] = true;

        for index in (0..self.nodes.len()).rev() {
            if needed[index] {
                for input in &self.nodes[index].inputs {
                    if let Input::Result(input) = *input {
                        needed[input] = true;
                    }
                }
            }
        }

        for index in (0..self.nodes.len()).filter(|&index| needed[index]) {
            self.plan_node(index);
        }

        self.convert(last, ColorSpace::Srgb)
    }

    fn plan_node(&mut self, index: usize) {
        let node = &self.nodes[index];
        let subregion = self.transform.covered_pixels(self.subregions[index]);
        let slots: Vec<usize> = node
            .inputs
            .iter()
            .map(|&input| self.plan_input(input))
            .collect();
        let first = slots.first().map(|&slot| self.outputs[slot].unwrap());

        // `feOffset` and `feTile` move pixels without changing them, so their result stays in the
        // color space of their input (§10).
        let space = match (&node.primitive, first) {
            (Primitive::Flood(_), _) => ColorSpace::Srgb,
            (Primitive::Offset(_) | Primitive::Tile(_), Some(first)) => first.space,
            _ => match node.color_interpolation {
                ColorInterpolation::LinearRgb => ColorSpace::LinearRgb,
                ColorInterpolation::Srgb | ColorInterpolation::Auto => ColorSpace::Srgb,
            },
        };

        let inputs: Vec<ImageId> = slots
            .iter()
            .map(|&slot| self.convert(slot, space))
            .collect();
        let dst = self.image(subregion.intersect(&self.region));
        let step = match (&node.primitive, inputs.as_slice()) {
            (Primitive::Flood(flood), []) => Step::Flood {
                dst,
                color: flood
                    .color
                    .with_opacity(flood.opacity)
                    .to_premultiplied_rgba8(),
            },
            (Primitive::Offset(offset), &[src]) => Step::Offset {
                src,
                dst,
                dx: (offset.dx * self.transform.sx).round() as i64,
                dy: (offset.dy * self.transform.sy).round() as i64,
            },
            (Primitive::Tile(_), &[src]) => Step::Tile {
                src,
                cell: first.unwrap().subregion,
                dst,
            },
            (Primitive::Merge(_), srcs) => Step::Merge {
                srcs: srcs.to_vec(),
                dst,
            },
            (Primitive::Composite(composite), &[src, backdrop]) => Step::Composite {
                src,
                backdrop,
                operator: composite.operator,
                dst,
            },
            (Primitive::Blend(blend), &[src, backdrop]) => Step::Blend {
                src,
                backdrop,
                mode: blend.mode,
                composite: !blend.no_composite,
                dst,
            },
            // A radius that isn't positive disables the primitive, which passes its input on
            // (§9.17). This includes a zero in only one of the two radii, which browsers instead
            // treat as a window of one pixel along that axis.
            (Primitive::Morphology(morphology), &[src])
                if !(morphology.radius_x > 0.0 && morphology.radius_y > 0.0) =>
            {
                Step::Offset {
                    src,
                    dst,
                    dx: 0,
                    dy: 0,
                }
            }
            (Primitive::Morphology(morphology), &[src]) => Step::Morphology {
                src,
                dst,
                operator: morphology.operator,
                rx: (morphology.radius_x * self.transform.sx).round() as u32,
                ry: (morphology.radius_y * self.transform.sy).round() as u32,
            },
            (Primitive::ColorMatrix(color_matrix), &[src]) => Step::ColorMatrix {
                src,
                dst,
                matrix: matrix(color_matrix),
            },
            (Primitive::ComponentTransfer(transfer), &[src]) => Step::ComponentTransfer {
                src,
                dst,
                functions: [
                    regions(&transfer.red),
                    regions(&transfer.green),
                    regions(&transfer.blue),
                    regions(&transfer.alpha),
                ],
            },
            // TODO: The remaining primitives.
            _ => Step::Flood { dst, color: [0; 4] },
        };

        self.steps.push(step);
        self.outputs[index] = Some(Output {
            image: dst,
            space,
            converted: None,
            subregion,
        });
    }

    fn plan_input(&mut self, input: Input) -> usize {
        let (slot, step): (usize, fn(ImageId) -> Step) = match input {
            Input::Result(index) => return index,
            Input::SourceGraphic => (self.nodes.len(), |dst| Step::SourceGraphic { dst }),
            Input::SourceAlpha => (self.nodes.len() + 1, |dst| Step::SourceAlpha { dst }),
        };

        if self.outputs[slot].is_none() {
            let dst = self.image(self.region);

            self.steps.push(step(dst));

            self.outputs[slot] = Some(Output {
                image: dst,
                space: ColorSpace::Srgb,
                converted: None,
                subregion: self.region,
            });
        }

        slot
    }

    fn convert(&mut self, slot: usize, space: ColorSpace) -> ImageId {
        let output = self.outputs[slot].unwrap();

        if output.space == space {
            return output.image;
        }

        if let Some(converted) = output.converted {
            return converted;
        }

        let dst = self.image(self.rects[output.image.0]);
        self.steps.push(Step::Convert {
            src: output.image,
            dst,
            from: output.space,
            to: space,
        });
        self.outputs[slot].as_mut().unwrap().converted = Some(dst);

        dst
    }

    fn image(&mut self, rect: IntRect) -> ImageId {
        self.rects.push(rect);

        ImageId(self.rects.len() - 1)
    }
}

impl Transform {
    fn covered_pixels(self, rect: Rect) -> IntRect {
        let Self { sx, sy, tx, ty } = self;

        if !(rect.width > 0.0 && rect.height > 0.0 && sx > 0.0 && sy > 0.0) {
            return IntRect::default();
        }

        IntRect::new(
            (rect.x * sx + tx).floor() as i32,
            (rect.y * sy + ty).floor() as i32,
            ((rect.x + rect.width) * sx + tx).ceil() as i32,
            ((rect.y + rect.height) * sy + ty).ceil() as i32,
        )
    }
}

fn matrix(color_matrix: &ColorMatrix) -> [f32; 20] {
    match *color_matrix {
        ColorMatrix::Matrix(matrix) => matrix,
        #[rustfmt::skip]
        ColorMatrix::Saturate(s) => [
            0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s, 0.0, 0.0,
            0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s, 0.0, 0.0,
            0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s, 0.0, 0.0,
            0.0, 0.0, 0.0, 1.0, 0.0,
        ],
        #[rustfmt::skip]
        ColorMatrix::HueRotate(degrees) => {
            let (sin, cos) = degrees.to_radians().sin_cos();
            let a = |constant: f32, cos_factor: f32, sin_factor: f32| {
                constant + cos * cos_factor + sin * sin_factor
            };

            [
                a(0.213, 0.787, -0.213), a(0.715, -0.715, -0.715), a(0.072, -0.072, 0.928), 0.0, 0.0,
                a(0.213, -0.213, 0.143), a(0.715, 0.285, 0.140), a(0.072, -0.072, -0.283), 0.0, 0.0,
                a(0.213, -0.213, -0.787), a(0.715, -0.715, 0.715), a(0.072, 0.928, 0.072), 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0, 0.0,
            ]
        }
        #[rustfmt::skip]
        ColorMatrix::LuminanceToAlpha => [
            0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            0.2126, 0.7152, 0.0722, 0.0, 0.0,
        ],
    }
}

fn regions(function: &TransferFunction) -> Vec<[f32; 2]> {
    match function {
        TransferFunction::Identity => vec![[0.0, 1.0]],
        TransferFunction::Table(values) | TransferFunction::Discrete(values)
            if values.is_empty() =>
        {
            vec![[0.0, 1.0]]
        }
        TransferFunction::Table(values) if values.len() == 1 => vec![[values[0], values[0]]],
        TransferFunction::Table(values) => {
            values.windows(2).map(|pair| [pair[0], pair[1]]).collect()
        }
        TransferFunction::Discrete(values) => values.iter().map(|&value| [value, value]).collect(),
        TransferFunction::Linear { slope, intercept } => vec![[*intercept, slope + intercept]],
        TransferFunction::Gamma {
            amplitude,
            exponent,
            offset,
        } => {
            let gamma = |i: u8| amplitude * (f32::from(i) / 255.0).powf(*exponent) + offset;

            (0..255).map(|i| [gamma(i), gamma(i + 1)]).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use filtra_types::{Composite, Flood, Merge, Offset, Tile};

    use super::*;
    use crate::Pass;

    const BOUNDS: IntRect = IntRect::new(0, 0, 100, 100);
    const IDENTITY: Transform = Transform::new(1.0, 1.0, 0.0, 0.0);

    #[test]
    fn connects_inputs() {
        let mut filter = filter();
        let shifted = filter.push(srgb(offset(), [Input::SourceGraphic]));
        let flooded = filter.push(srgb(flood(), []));
        let composited = filter.push(srgb(
            Primitive::Composite(Composite::default()),
            [shifted, flooded],
        ));
        filter.push(srgb(
            Primitive::Merge(Merge),
            [Input::SourceAlpha, Input::SourceAlpha, composited],
        ));

        let plan = plan(&filter, IDENTITY, BOUNDS);
        let source_graphic = single(&plan, |step| matches!(step, Step::SourceGraphic { .. }));
        let source_alpha = single(&plan, |step| matches!(step, Step::SourceAlpha { .. }));
        let flooded = single(&plan, |step| matches!(step, Step::Flood { .. }));

        let Step::Offset {
            src, dst: shifted, ..
        } = *single(&plan, |step| matches!(step, Step::Offset { .. }))
        else {
            unreachable!()
        };
        assert_eq!(src, source_graphic.writes());

        let Step::Composite {
            src,
            backdrop,
            dst: composited,
            ..
        } = *single(&plan, |step| matches!(step, Step::Composite { .. }))
        else {
            unreachable!()
        };
        assert_eq!((src, backdrop), (shifted, flooded.writes()));

        let Step::Merge { srcs, dst } = single(&plan, |step| matches!(step, Step::Merge { .. }))
        else {
            unreachable!()
        };
        let alpha = source_alpha.writes();
        assert_eq!(*srcs, [alpha, alpha, composited]);
        assert_eq!(plan.output, *dst);
    }

    #[test]
    fn skips_unused_nodes_and_inputs() {
        let mut filter = filter();
        filter.push(srgb(flood(), []));
        filter.push(srgb(offset(), [Input::SourceGraphic]));

        let plan = plan(&filter, IDENTITY, BOUNDS);
        assert!(
            !plan
                .steps
                .iter()
                .any(|step| matches!(step, Step::Flood { .. } | Step::SourceAlpha { .. }))
        );
    }

    #[test]
    fn converts_each_value_once_between_spaces() {
        let mut filter = filter();
        let mut node = Node::new(
            Primitive::Merge(Merge),
            [Input::SourceGraphic, Input::SourceGraphic],
        );
        node.color_interpolation = ColorInterpolation::LinearRgb;
        let merged = filter.push(node);
        filter.push(srgb(
            Primitive::Merge(Merge),
            [merged, Input::SourceGraphic],
        ));

        // `SourceGraphic` is converted once for both reads and read as it is in sRGB, and the
        // sRGB result needs no final conversion.
        let plan = plan(&filter, IDENTITY, BOUNDS);
        let conversions: Vec<(ColorSpace, ColorSpace)> = (plan.steps.iter())
            .filter_map(|step| match *step {
                Step::Convert { from, to, .. } => Some((from, to)),
                _ => None,
            })
            .collect();

        assert_eq!(
            conversions,
            [
                (ColorSpace::Srgb, ColorSpace::LinearRgb),
                (ColorSpace::LinearRgb, ColorSpace::Srgb),
            ]
        );
    }

    #[test]
    fn offset_tile_and_flood_keep_their_space() {
        let inputs = [
            vec![Input::SourceGraphic],
            vec![Input::SourceGraphic],
            vec![],
        ];
        let primitives = [offset(), Primitive::Tile(Tile), flood()];

        for (primitive, inputs) in primitives.into_iter().zip(inputs) {
            let mut filter = filter();
            filter.push(Node::new(primitive, inputs));

            let plan = plan(&filter, IDENTITY, BOUNDS);
            assert!(
                !plan
                    .steps
                    .iter()
                    .any(|step| matches!(step, Step::Convert { .. }))
            );
        }
    }

    #[test]
    fn transparent_results_are_empty() {
        let mut empty_region = Filter::new(Rect::new(0.0, 0.0, 0.0, 100.0));
        empty_region.push(srgb(offset(), [Input::SourceGraphic]));

        let mut chain = filter();
        chain.push(srgb(offset(), [Input::SourceGraphic]));

        for (filter, transform) in [
            (&filter(), IDENTITY),
            (&empty_region, IDENTITY),
            (&chain, Transform::new(0.0, 1.0, 0.0, 0.0)),
        ] {
            let plan = plan(filter, transform, BOUNDS);
            assert!(plan.rects[plan.output.0].is_empty());
        }
    }

    fn filter() -> Filter {
        Filter::new(Rect::new(0.0, 0.0, 100.0, 100.0))
    }

    fn flood() -> Primitive {
        Primitive::Flood(Flood::default())
    }

    fn offset() -> Primitive {
        Primitive::Offset(Offset { dx: 1.0, dy: 1.0 })
    }

    fn srgb(primitive: Primitive, inputs: impl IntoIterator<Item = Input>) -> Node {
        let mut node = Node::new(primitive, inputs);
        node.color_interpolation = ColorInterpolation::Srgb;

        node
    }

    fn single(plan: &Plan, kind: impl Fn(&Step) -> bool) -> &Step {
        let mut steps = plan.steps.iter().filter(|step| kind(step));
        let step = steps.next().unwrap();

        assert!(steps.next().is_none());

        step
    }
}
