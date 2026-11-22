use alloc::vec::Vec;

use crate::FilterFunction;
use crate::{ColorInterpolation, Primitive};

/// A `filter` element: its region and its primitives, with `I` the content of `feImage` nodes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// Deserializing goes through `push`, which keeps every reference pointing at an earlier node.
#[cfg_attr(feature = "serde", serde(from = "Nodes<I>"))]
pub struct Filter<I> {
    /// The filter region in user space.
    pub region: Rect,
    nodes: Vec<Node<I>>,
}

impl<I> Filter<I> {
    /// Creates a filter with the given region and no primitives.
    pub fn new(region: Rect) -> Self {
        Self {
            region,
            nodes: Vec::new(),
        }
    }

    /// Appends a node and returns the input that refers to its result.
    pub fn push(&mut self, mut node: Node<I>) -> Input {
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

    /// The nodes in document order.
    pub fn nodes(&self) -> &[Node<I>] {
        &self.nodes
    }

    /// Converts from `&Filter<I>` to `Filter<&I>`.
    pub fn as_ref(&self) -> Filter<&I> {
        let nodes = self.nodes.iter().map(|node| Node {
            primitive: node.primitive.as_ref(),
            inputs: node.inputs.clone(),
            subregion: node.subregion,
            color_interpolation: node.color_interpolation,
        });

        Filter {
            region: self.region,
            nodes: nodes.collect(),
        }
    }

    /// The filter with the content of each `feImage` mapped by `f`.
    pub fn try_map_images<J, E>(
        self,
        mut f: impl FnMut(I) -> Result<J, E>,
    ) -> Result<Filter<J>, E> {
        let nodes = self.nodes.into_iter().map(|node| {
            Ok(Node {
                primitive: node.primitive.try_map_image(&mut f)?,
                inputs: node.inputs,
                subregion: node.subregion,
                color_interpolation: node.color_interpolation,
            })
        });

        Ok(Filter {
            region: self.region,
            nodes: nodes.collect::<Result<_, E>>()?,
        })
    }

    /// The filter primitive subregion of each node, with missing attributes filled in (§9.4).
    pub fn subregions(&self) -> Vec<Rect> {
        let mut subregions: Vec<Rect> = Vec::with_capacity(self.nodes.len());

        for node in &self.nodes {
            // Missing attributes default to the union of the input subregions, or to the filter
            // region for nodes without inputs, with a standard input, and for `feTile`.
            let uses_region = matches!(node.primitive, Primitive::Tile(_))
                || node
                    .inputs
                    .iter()
                    .any(|input| matches!(input, Input::SourceGraphic | Input::SourceAlpha));

            let default = node
                .inputs
                .iter()
                .filter_map(|input| match input {
                    Input::Result(index) => Some(subregions[*index]),
                    _ => None,
                })
                .reduce(Rect::union)
                .filter(|_| !uses_region)
                .unwrap_or(self.region);

            subregions.push(node.subregion.resolve(default));
        }

        subregions
    }

    /// The filter equivalent to a list of filter functions applied to content within `bounds`.
    pub fn from_functions(functions: &[FilterFunction], bounds: Rect) -> Self {
        let mut filter = Self::new(bounds);
        let mut input = Input::SourceGraphic;
        for function in functions {
            filter.region = function.extend(filter.region);
            let mut node = Node::new(function.to_primitive(), [input]);
            node.color_interpolation = ColorInterpolation::Srgb;
            input = filter.push(node);
        }
        filter
    }
}

/// A filter primitive with its inputs and common attributes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node<I> {
    /// The primitive.
    pub primitive: Primitive<I>,
    /// The inputs in the order the primitive takes them (`in`, `in2` or the merge nodes).
    pub inputs: Vec<Input>,
    /// The filter primitive subregion.
    pub subregion: Subregion,
    /// The `color-interpolation-filters` property.
    pub color_interpolation: ColorInterpolation,
}

impl<I> Node<I> {
    /// Creates a node without subregion attributes and with the initial color interpolation.
    pub fn new(primitive: Primitive<I>, inputs: impl IntoIterator<Item = Input>) -> Self {
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Input {
    /// The filtered element.
    SourceGraphic,
    /// The alpha channel of the filtered element, with black color.
    SourceAlpha,
    /// The result of an earlier node of the same filter, by index.
    Result(usize),
}

/// The `x`, `y`, `width` and `height` attributes of a filter primitive in user space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl Subregion {
    fn resolve(self, default: Rect) -> Rect {
        Rect::new(
            self.x.unwrap_or(default.x),
            self.y.unwrap_or(default.y),
            self.width.unwrap_or(default.width),
            self.height.unwrap_or(default.height),
        )
    }
}

/// A rectangle in user space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub(crate) fn outset(self, amount: f32) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.width + 2.0 * amount,
            self.height + 2.0 * amount,
        )
    }

    pub(crate) fn union(self, other: Self) -> Self {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.width).max(other.x + other.width);
        let y1 = (self.y + self.height).max(other.y + other.height);
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct Nodes<I> {
    region: Rect,
    nodes: Vec<Node<I>>,
}

#[cfg(feature = "serde")]
impl<I> From<Nodes<I>> for Filter<I> {
    fn from(Nodes { region, nodes }: Nodes<I>) -> Self {
        let mut filter = Self::new(region);

        for node in nodes {
            filter.push(node);
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Offset;
    use alloc::string::{String, ToString};

    #[test]
    fn maps_images_and_keeps_the_rest() {
        let mut filter = Filter::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        let image = filter.push(Node::new(Primitive::Image(1), []));
        let offset = Offset { dx: 5.0, dy: 5.0 };
        filter.push(Node::new(Primitive::Offset(offset), [image]));
        filter.push(Node::new(Primitive::Image(2), []));

        let mapped = filter
            .try_map_images(|id| Ok::<_, ()>(id.to_string()))
            .unwrap();

        let primitives: Vec<&Primitive<String>> =
            mapped.nodes().iter().map(|node| &node.primitive).collect();
        assert_eq!(
            primitives,
            [
                &Primitive::Image("1".to_string()),
                &Primitive::Offset(offset),
                &Primitive::Image("2".to_string()),
            ]
        );
        assert_eq!(mapped.nodes()[1].inputs, [Input::Result(0)]);
    }

    #[test]
    fn mapping_images_stops_at_the_first_error() {
        let mut filter = Filter::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        filter.push(Node::new(Primitive::Image(1), []));
        filter.push(Node::new(Primitive::Image(2), []));
        filter.push(Node::new(Primitive::Image(3), []));
        let mut visited = Vec::new();

        let result = filter.as_ref().try_map_images(|&id| {
            visited.push(id);
            if id == 2 { Err(id) } else { Ok(id) }
        });

        assert_eq!(result, Err(2));
        assert_eq!(visited, [1, 2]);
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
    use crate::{Merge, Offset};

    #[test]
    fn serde_round_trip() {
        let filter = shadow();
        let json = serde_json::to_string(&filter).unwrap();

        assert_eq!(serde_json::from_str::<Filter<()>>(&json).unwrap(), filter);
    }

    #[test]
    fn deserializing_redirects_references_to_later_nodes() {
        let mut json = serde_json::to_value(shadow()).unwrap();
        json["nodes"][0]["inputs"][0] = serde_json::to_value(Input::Result(2)).unwrap();
        let filter: Filter<()> = serde_json::from_value(json).unwrap();

        assert_eq!(filter.nodes()[0].inputs, [Input::SourceGraphic]);
    }

    fn shadow() -> Filter<()> {
        let mut filter = Filter::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        let offset = Offset { dx: 5.0, dy: 5.0 };
        let shadow = filter.push(Node::new(Primitive::Offset(offset), [Input::SourceAlpha]));
        let merge = Node::new(Primitive::Merge(Merge), [shadow, Input::SourceGraphic]);
        filter.push(merge);

        filter
    }
}
