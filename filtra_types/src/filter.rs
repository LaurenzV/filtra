use alloc::vec::Vec;

use crate::{ColorInterpolation, FilterFunction, Primitive};

/// A `filter` element: its region and its primitives.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    /// The filter region in user space.
    pub region: Rect,
    nodes: Vec<Node>,
}

impl Filter {
    /// Creates a filter with the given region and no primitives.
    pub fn new(region: Rect) -> Self {
        Self {
            region,
            nodes: Vec::new(),
        }
    }

    /// Appends a node and returns the input that refers to its result.
    pub fn push(&mut self, mut node: Node) -> Input {
        let count = self.nodes.len();
        for input in &mut node.inputs {
            // A reference to a node that doesn't exist yet acts like a missing `in` (§9.2).
            if matches!(*input, Input::Result(index) if index >= count) {
                *input = match count {
                    0 => Input::SourceGraphic,
                    _ => Input::Result(count - 1),
                };
            }
        }
        self.nodes.push(node);
        Input::Result(count)
    }

    /// The primitives in document order; the result of the last one is the filter's result.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// The filter equivalent to a list of filter functions applied to content within `bounds`.
    pub fn from_functions(functions: &[FilterFunction], bounds: Rect) -> Self {
        let mut filter = Self::new(bounds);
        let mut input = Input::SourceGraphic;
        for function in functions {
            // A blur visibly extends the content by about 3 standard deviations on each side.
            match *function {
                FilterFunction::Blur(std_dev) => {
                    filter.region = filter.region.outset(3.0 * std_dev.max(0.0));
                }
                FilterFunction::DropShadow {
                    dx, dy, std_dev, ..
                } => {
                    let mut shadow = filter.region.outset(3.0 * std_dev.max(0.0));
                    shadow.x += dx;
                    shadow.y += dy;
                    filter.region = filter.region.union(shadow);
                }
                _ => {}
            }
            let mut node = Node::new(function.to_primitive(), [input]);
            node.color_interpolation = ColorInterpolation::Srgb;
            input = filter.push(node);
        }
        filter
    }
}

/// A filter primitive with its inputs and common attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// The primitive.
    pub primitive: Primitive,
    /// The inputs in the order the primitive takes them (`in`, `in2` or the merge nodes).
    pub inputs: Vec<Input>,
    /// The filter primitive subregion.
    pub subregion: Subregion,
    /// The `color-interpolation-filters` property.
    pub color_interpolation: ColorInterpolation,
}

impl Node {
    /// Creates a node without subregion attributes and with the initial color interpolation.
    pub fn new(primitive: Primitive, inputs: impl IntoIterator<Item = Input>) -> Self {
        Self {
            primitive,
            inputs: inputs.into_iter().collect(),
            subregion: Subregion::default(),
            color_interpolation: ColorInterpolation::default(),
        }
    }
}

/// An input of a filter primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    /// The filtered element.
    SourceGraphic,
    /// The alpha channel of the filtered element, with black color.
    SourceAlpha,
    /// The result of an earlier node of the same filter, by index.
    Result(usize),
    // TODO: `BackgroundImage`, `BackgroundAlpha`, `FillPaint` and `StrokePaint`.
}

/// The `x`, `y`, `width` and `height` attributes of a filter primitive in user space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Subregion {
    /// The `x` attribute.
    pub x: Option<f32>,
    /// The `y` attribute.
    pub y: Option<f32>,
    /// The `width` attribute.
    pub width: Option<f32>,
    /// The `height` attribute.
    pub height: Option<f32>,
}

/// A rectangle in user space, empty if its width or height is zero or negative.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// The left edge.
    pub x: f32,
    /// The top edge.
    pub y: f32,
    /// The width.
    pub width: f32,
    /// The height.
    pub height: f32,
}

impl Rect {
    /// Creates a rectangle from its position and size.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn outset(self, amount: f32) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.width + 2.0 * amount,
            self.height + 2.0 * amount,
        )
    }

    fn union(self, other: Self) -> Self {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.width).max(other.x + other.width);
        let y1 = (self.y + self.height).max(other.y + other.height);
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }
}
