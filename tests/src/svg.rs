//! A parser for the minimal SVG subset of the test files: shapes with solid fills in one `<g>`,
//! filtered either by one `<filter>` in user space or by a list of filter functions.

use filtra::types::{
    Blend, BlendMode, Color, ColorChannel, ColorInterpolation, ColorMatrix, ComponentTransfer,
    Composite, CompositeOperator, ConvolveMatrix, DiffuseLighting, DisplacementMap, DistantLight,
    DropShadow, EdgeMode, Filter, FilterFunction, Flood, GaussianBlur, Input, LightSource, Merge,
    Morphology, MorphologyOperator, Node, Offset, PointLight, Primitive, Rect, SpecularLighting,
    SpotLight, Subregion, Tile, TransferFunction, Turbulence, TurbulenceType,
};
use roxmltree::Node as XmlNode;
use vello_cpu::kurbo::{self, BezPath, Circle, Shape as _};

pub struct Document {
    pub width: u32,
    pub height: u32,
    pub shapes: Vec<Shape>,
    pub filter: Option<Filter<Image>>,
}

pub type Image = Vec<Shape>;

pub struct Shape {
    pub path: BezPath,
    pub color: Color,
}

pub fn parse(text: &str) -> Document {
    let tree = roxmltree::Document::parse(text).expect("invalid XML");
    let root = tree.root_element();
    let shapes: Vec<Shape> = (root.descendants())
        .filter(|node| {
            !node
                .ancestors()
                .any(|ancestor| ancestor.has_tag_name("defs"))
        })
        .filter_map(shape)
        .collect();
    let group = root.descendants().find(|node| node.has_tag_name("g"));
    let element = root.descendants().find(|node| node.has_tag_name("filter"));

    let filter = group.map(|group| match element {
        Some(element) => filter(element),
        None => filter_functions(attribute(group, "filter"), &shapes),
    });

    Document {
        width: number(root, "width") as u32,
        height: number(root, "height") as u32,
        shapes,
        filter,
    }
}

fn shape(node: XmlNode<'_, '_>) -> Option<Shape> {
    let number = |name| f64::from(number(node, name));
    let path = match node.tag_name().name() {
        "rect" => {
            let (x, y) = (number("x"), number("y"));
            kurbo::Rect::new(x, y, x + number("width"), y + number("height")).to_path(0.1)
        }
        "circle" => Circle::new((number("cx"), number("cy")), number("r")).to_path(0.1),
        "path" => BezPath::from_svg(attribute(node, "d")).expect("invalid path"),
        _ => return None,
    };

    let opacity = number_or(node, "fill-opacity", 1.0);
    let color = color(attribute(node, "fill")).with_opacity(opacity);

    Some(Shape { path, color })
}

fn filter(element: XmlNode<'_, '_>) -> Filter<Image> {
    let region = Rect::new(
        number(element, "x"),
        number(element, "y"),
        number(element, "width"),
        number(element, "height"),
    );
    let interpolation = color_interpolation(element).unwrap_or(ColorInterpolation::LinearRgb);
    let mut filter = Filter::new(region);
    let mut results = Vec::new();

    for child in element.children().filter(XmlNode::is_element) {
        let input = |name| resolve_input(child, name, &results);
        let (primitive, inputs) = match child.tag_name().name() {
            "feBlend" => {
                let blend = Blend {
                    mode: blend_mode(child),
                    no_composite: child.has_attribute("no-composite"),
                };
                (Primitive::Blend(blend), vec![input("in"), input("in2")])
            }
            "feColorMatrix" => (
                Primitive::ColorMatrix(color_matrix(child)),
                vec![input("in")],
            ),
            "feComponentTransfer" => {
                let transfer = component_transfer(child);
                (Primitive::ComponentTransfer(transfer), vec![input("in")])
            }
            "feComposite" => {
                let composite = Composite {
                    operator: composite_operator(child),
                };
                (
                    Primitive::Composite(composite),
                    vec![input("in"), input("in2")],
                )
            }
            "feConvolveMatrix" => (
                Primitive::ConvolveMatrix(convolve_matrix(child)),
                vec![input("in")],
            ),
            "feDiffuseLighting" => {
                let mut diffuse = DiffuseLighting::new(light_source(child));
                diffuse.surface_scale = number_or(child, "surfaceScale", 1.0);
                diffuse.diffuse_constant = number_or(child, "diffuseConstant", 1.0);
                diffuse.lighting_color = child
                    .attribute("lighting-color")
                    .map_or(Color::WHITE, color);
                (Primitive::DiffuseLighting(diffuse), vec![input("in")])
            }
            "feSpecularLighting" => {
                let mut specular = SpecularLighting::new(light_source(child));
                specular.surface_scale = number_or(child, "surfaceScale", 1.0);
                specular.specular_constant = number_or(child, "specularConstant", 1.0);
                specular.specular_exponent = number_or(child, "specularExponent", 1.0);
                specular.lighting_color = child
                    .attribute("lighting-color")
                    .map_or(Color::WHITE, color);
                (Primitive::SpecularLighting(specular), vec![input("in")])
            }
            "feDropShadow" => {
                let (std_dev_x, std_dev_y) = std_deviation(child, 2.0);
                let shadow = DropShadow {
                    dx: number_or(child, "dx", 2.0),
                    dy: number_or(child, "dy", 2.0),
                    std_dev_x,
                    std_dev_y,
                    color: child.attribute("flood-color").map_or(Color::BLACK, color),
                    opacity: number_or(child, "flood-opacity", 1.0),
                };
                (Primitive::DropShadow(shadow), vec![input("in")])
            }
            "feFlood" => {
                let flood = Flood {
                    color: child.attribute("flood-color").map_or(Color::BLACK, color),
                    opacity: number_or(child, "flood-opacity", 1.0),
                };
                (Primitive::Flood(flood), vec![])
            }
            "feGaussianBlur" => (
                Primitive::GaussianBlur(gaussian_blur(child)),
                vec![input("in")],
            ),
            "feImage" => {
                let id = attribute(child, "href")
                    .strip_prefix('#')
                    .expect("feImage only references elements");
                let referenced = (child.document().descendants())
                    .find(|node| node.attribute("id") == Some(id))
                    .unwrap_or_else(|| panic!("no element with id {id}"));
                let image = referenced.descendants().filter_map(shape).collect();
                (Primitive::Image(image), vec![])
            }
            "feMerge" => {
                let nodes = child.children().filter(XmlNode::is_element);
                let inputs = nodes.map(|node| resolve_input(node, "in", &results));
                (Primitive::Merge(Merge), inputs.collect())
            }
            "feDisplacementMap" => {
                let displacement = DisplacementMap {
                    scale: number_or(child, "scale", 0.0),
                    x_channel: color_channel(child, "xChannelSelector"),
                    y_channel: color_channel(child, "yChannelSelector"),
                };
                (
                    Primitive::DisplacementMap(displacement),
                    vec![input("in"), input("in2")],
                )
            }
            "feMorphology" => {
                let morphology = morphology(child);
                (Primitive::Morphology(morphology), vec![input("in")])
            }
            "feOffset" => {
                let offset = Offset {
                    dx: number_or(child, "dx", 0.0),
                    dy: number_or(child, "dy", 0.0),
                };
                (Primitive::Offset(offset), vec![input("in")])
            }
            "feTile" => (Primitive::Tile(Tile), vec![input("in")]),
            "feTurbulence" => (Primitive::Turbulence(turbulence(child)), vec![]),
            other => panic!("unsupported filter primitive <{other}>"),
        };

        let mut node = Node::new(primitive, inputs);
        node.subregion = Subregion {
            x: optional_number(child, "x"),
            y: optional_number(child, "y"),
            width: optional_number(child, "width"),
            height: optional_number(child, "height"),
        };
        node.color_interpolation = color_interpolation(child).unwrap_or(interpolation);

        filter.push(node);
        results.push(child.attribute("result"));
    }

    filter
}

/// A missing `in` or a reference to an unknown result reads the previous result, or
/// `SourceGraphic` for the first primitive (§9.2).
fn resolve_input(node: XmlNode<'_, '_>, name: &str, results: &[Option<&str>]) -> Input {
    let previous = match results.len() {
        0 => Input::SourceGraphic,
        n => Input::Result(n - 1),
    };

    match node.attribute(name) {
        None => previous,
        Some("SourceGraphic") => Input::SourceGraphic,
        Some("SourceAlpha") => Input::SourceAlpha,
        Some(reference) => results
            .iter()
            .rposition(|result| *result == Some(reference))
            .map_or(previous, Input::Result),
    }
}

fn color_interpolation(node: XmlNode<'_, '_>) -> Option<ColorInterpolation> {
    node.attribute("color-interpolation-filters")
        .map(|value| match value {
            "auto" => ColorInterpolation::Auto,
            "sRGB" => ColorInterpolation::Srgb,
            "linearRGB" => ColorInterpolation::LinearRgb,
            other => panic!("invalid color-interpolation-filters \"{other}\""),
        })
}

fn blend_mode(node: XmlNode<'_, '_>) -> BlendMode {
    match node.attribute("mode").unwrap_or("normal") {
        "normal" => BlendMode::Normal,
        "multiply" => BlendMode::Multiply,
        "screen" => BlendMode::Screen,
        "overlay" => BlendMode::Overlay,
        "darken" => BlendMode::Darken,
        "lighten" => BlendMode::Lighten,
        "color-dodge" => BlendMode::ColorDodge,
        "color-burn" => BlendMode::ColorBurn,
        "hard-light" => BlendMode::HardLight,
        "soft-light" => BlendMode::SoftLight,
        "difference" => BlendMode::Difference,
        "exclusion" => BlendMode::Exclusion,
        "hue" => BlendMode::Hue,
        "saturation" => BlendMode::Saturation,
        "color" => BlendMode::Color,
        "luminosity" => BlendMode::Luminosity,
        other => panic!("unsupported feBlend mode \"{other}\""),
    }
}

fn color_matrix(node: XmlNode<'_, '_>) -> ColorMatrix {
    let values = node.attribute("values").map(numbers);
    // `values` with the wrong number of entries makes the primitive a pass-through (§9.6).
    let identity = ColorMatrix::Matrix(ColorMatrix::IDENTITY);

    match (
        node.attribute("type").unwrap_or("matrix"),
        values.as_deref(),
    ) {
        ("matrix", None) => identity,
        ("matrix", Some(values)) => values.try_into().map_or(identity, ColorMatrix::Matrix),
        ("saturate", None) => ColorMatrix::Saturate(1.0),
        ("saturate", Some(&[value])) => ColorMatrix::Saturate(value),
        ("hueRotate", None) => ColorMatrix::HueRotate(0.0),
        ("hueRotate", Some(&[value])) => ColorMatrix::HueRotate(value),
        ("saturate" | "hueRotate", Some(_)) => identity,
        ("luminanceToAlpha", _) => ColorMatrix::LuminanceToAlpha,
        (other, _) => panic!("unsupported feColorMatrix type \"{other}\""),
    }
}

/// `radius` with one number for both axes or two; anything else is the initial 0 (§9.17).
fn color_channel(node: XmlNode<'_, '_>, name: &str) -> ColorChannel {
    match node.attribute(name).unwrap_or("A") {
        "R" => ColorChannel::R,
        "G" => ColorChannel::G,
        "B" => ColorChannel::B,
        "A" => ColorChannel::A,
        other => panic!("unsupported {name} \"{other}\""),
    }
}

fn morphology(node: XmlNode<'_, '_>) -> Morphology {
    let radius = node.attribute("radius").map(numbers).unwrap_or_default();
    let (radius_x, radius_y) = match *radius.as_slice() {
        [r] => (r, r),
        [x, y] => (x, y),
        _ => (0.0, 0.0),
    };
    let operator = match node.attribute("operator").unwrap_or("erode") {
        "erode" => MorphologyOperator::Erode,
        "dilate" => MorphologyOperator::Dilate,
        other => panic!("unsupported feMorphology operator \"{other}\""),
    };

    Morphology {
        operator,
        radius_x,
        radius_y,
    }
}

fn turbulence(node: XmlNode<'_, '_>) -> Turbulence {
    let frequency = node
        .attribute("baseFrequency")
        .map(numbers)
        .unwrap_or_default();
    let (base_frequency_x, base_frequency_y) = match *frequency.as_slice() {
        [f] => (f, f),
        [x, y] => (x, y),
        _ => (0.0, 0.0),
    };
    let kind = match node.attribute("type").unwrap_or("turbulence") {
        "turbulence" => TurbulenceType::Turbulence,
        "fractalNoise" => TurbulenceType::FractalNoise,
        other => panic!("unsupported feTurbulence type \"{other}\""),
    };

    Turbulence {
        base_frequency_x,
        base_frequency_y,
        num_octaves: node.attribute("numOctaves").map_or(1, |value| {
            value.trim().parse().expect("numOctaves is not an integer")
        }),
        seed: number_or(node, "seed", 0.0),
        stitch_tiles: node.attribute("stitchTiles") == Some("stitch"),
        kind,
    }
}

fn component_transfer(node: XmlNode<'_, '_>) -> ComponentTransfer {
    let mut transfer = ComponentTransfer::default();

    for child in node.children().filter(XmlNode::is_element) {
        let table = || {
            child
                .attribute("tableValues")
                .map(numbers)
                .unwrap_or_default()
        };
        let function = match attribute(child, "type") {
            "identity" => TransferFunction::Identity,
            "table" => TransferFunction::Table(table()),
            "discrete" => TransferFunction::Discrete(table()),
            "linear" => TransferFunction::Linear {
                slope: number_or(child, "slope", 1.0),
                intercept: number_or(child, "intercept", 0.0),
            },
            "gamma" => TransferFunction::Gamma {
                amplitude: number_or(child, "amplitude", 1.0),
                exponent: number_or(child, "exponent", 1.0),
                offset: number_or(child, "offset", 0.0),
            },
            other => panic!("unsupported transfer function type \"{other}\""),
        };

        match child.tag_name().name() {
            "feFuncR" => transfer.red = function,
            "feFuncG" => transfer.green = function,
            "feFuncB" => transfer.blue = function,
            _ => transfer.alpha = function,
        }
    }

    transfer
}

fn gaussian_blur(node: XmlNode<'_, '_>) -> GaussianBlur {
    let (std_dev_x, std_dev_y) = std_deviation(node, 0.0);
    let edge_mode = match node.attribute("edgeMode").unwrap_or("none") {
        "duplicate" => EdgeMode::Duplicate,
        "wrap" => EdgeMode::Wrap,
        "mirror" => EdgeMode::Mirror,
        "none" => EdgeMode::None,
        other => panic!("invalid edgeMode \"{other}\""),
    };

    GaussianBlur {
        std_dev_x,
        std_dev_y,
        edge_mode,
    }
}

fn convolve_matrix(node: XmlNode<'_, '_>) -> ConvolveMatrix {
    // Orders are truncated (§9.9); 0 stands for anything below 1, which is unsupported.
    let order = |value: f32| if value >= 1.0 { value as u32 } else { 0 };
    let (order_x, order_y) = match node.attribute("order").map(numbers).as_deref() {
        Some(&[both]) => (order(both), order(both)),
        Some(&[x, y]) => (order(x), order(y)),
        Some(_) => (0, 0),
        None => (3, 3),
    };
    let integer = |name| {
        optional_number(node, name)
            .filter(|value| value.fract() == 0.0)
            .map(|value| value as i32)
    };
    let edge_mode = match node.attribute("edgeMode").unwrap_or("duplicate") {
        "duplicate" => EdgeMode::Duplicate,
        "wrap" => EdgeMode::Wrap,
        "mirror" => EdgeMode::Mirror,
        "none" => EdgeMode::None,
        other => panic!("invalid edgeMode \"{other}\""),
    };

    ConvolveMatrix {
        order_x,
        order_y,
        kernel: node
            .attribute("kernelMatrix")
            .map(numbers)
            .unwrap_or_default(),
        divisor: optional_number(node, "divisor"),
        bias: number_or(node, "bias", 0.0),
        target_x: integer("targetX"),
        target_y: integer("targetY"),
        edge_mode,
        kernel_unit_length: None,
        preserve_alpha: node.attribute("preserveAlpha") == Some("true"),
    }
}

fn light_source(node: XmlNode<'_, '_>) -> LightSource {
    let light = node
        .children()
        .find(XmlNode::is_element)
        .unwrap_or_else(|| panic!("<{}> has no light source", node.tag_name().name()));
    let number = |name| number_or(light, name, 0.0);

    match light.tag_name().name() {
        "feDistantLight" => LightSource::Distant(DistantLight {
            azimuth: number("azimuth"),
            elevation: number("elevation"),
        }),
        "fePointLight" => LightSource::Point(PointLight {
            x: number("x"),
            y: number("y"),
            z: number("z"),
        }),
        "feSpotLight" => LightSource::Spot(SpotLight {
            x: number("x"),
            y: number("y"),
            z: number("z"),
            points_at_x: number("pointsAtX"),
            points_at_y: number("pointsAtY"),
            points_at_z: number("pointsAtZ"),
            specular_exponent: number_or(light, "specularExponent", 1.0),
            limiting_cone_angle: optional_number(light, "limitingConeAngle"),
        }),
        other => panic!("unsupported light source <{other}>"),
    }
}

fn std_deviation(node: XmlNode<'_, '_>, initial: f32) -> (f32, f32) {
    // Anything but one or two numbers is an error, which leaves the initial value.
    match node.attribute("stdDeviation").map(numbers).as_deref() {
        Some(&[both]) => (both, both),
        Some(&[x, y]) => (x, y),
        _ => (initial, initial),
    }
}

fn composite_operator(node: XmlNode<'_, '_>) -> CompositeOperator {
    match node.attribute("operator").unwrap_or("over") {
        "over" => CompositeOperator::Over,
        "in" => CompositeOperator::In,
        "out" => CompositeOperator::Out,
        "atop" => CompositeOperator::Atop,
        "xor" => CompositeOperator::Xor,
        "lighter" => CompositeOperator::Lighter,
        "arithmetic" => CompositeOperator::Arithmetic {
            k1: number_or(node, "k1", 0.0),
            k2: number_or(node, "k2", 0.0),
            k3: number_or(node, "k3", 0.0),
            k4: number_or(node, "k4", 0.0),
        },
        other => panic!("unsupported feComposite operator \"{other}\""),
    }
}

fn filter_functions(value: &str, shapes: &[Shape]) -> Filter<Image> {
    let functions: Vec<FilterFunction> = value
        .split(')')
        .map(str::trim)
        .filter(|function| !function.is_empty())
        .map(|function| {
            let (name, argument) = function.split_once('(').expect("invalid filter function");
            let argument = argument.trim();

            match name.trim() {
                "blur" => FilterFunction::Blur(length(argument)),
                "brightness" => FilterFunction::Brightness(amount(argument)),
                "contrast" => FilterFunction::Contrast(amount(argument)),
                "drop-shadow" => drop_shadow(argument),
                "grayscale" => FilterFunction::Grayscale(amount(argument)),
                "hue-rotate" => FilterFunction::HueRotate(angle(argument)),
                "invert" => FilterFunction::Invert(amount(argument)),
                "opacity" => FilterFunction::Opacity(amount(argument)),
                "saturate" => FilterFunction::Saturate(amount(argument)),
                "sepia" => FilterFunction::Sepia(amount(argument)),
                other => panic!("unsupported filter function {other}()"),
            }
        })
        .collect();

    let bounds = shapes
        .iter()
        .map(|shape| shape.path.bounding_box())
        .reduce(|a, b| a.union(b))
        .unwrap_or_default();
    let bounds = Rect::new(
        bounds.x0 as f32,
        bounds.y0 as f32,
        bounds.width() as f32,
        bounds.height() as f32,
    );

    Filter::from_functions(&functions, bounds)
}

fn drop_shadow(argument: &str) -> FilterFunction {
    // The color defaults to `currentcolor`, which is black in the tests.
    let (colors, lengths): (Vec<&str>, Vec<&str>) = argument
        .split_whitespace()
        .partition(|part| part.starts_with('#'));
    let (dx, dy, std_dev) = match lengths.as_slice() {
        [dx, dy] => (length(dx), length(dy), 0.0),
        [dx, dy, std_dev] => (length(dx), length(dy), length(std_dev)),
        _ => panic!("invalid drop-shadow({argument})"),
    };

    FilterFunction::DropShadow {
        dx,
        dy,
        std_dev,
        color: colors.first().map_or(Color::BLACK, |value| color(value)),
    }
}

fn amount(argument: &str) -> f32 {
    match argument.strip_suffix('%') {
        _ if argument.is_empty() => 1.0,
        Some(percentage) => parse_number(percentage) / 100.0,
        None => parse_number(argument),
    }
}

fn length(argument: &str) -> f32 {
    match argument {
        "" => 0.0,
        _ => parse_number(argument.trim_end_matches("px")),
    }
}

fn angle(argument: &str) -> f32 {
    match argument.strip_suffix("turn") {
        _ if argument.is_empty() => 0.0,
        Some(turns) => parse_number(turns) * 360.0,
        None => parse_number(argument.trim_end_matches("deg")),
    }
}

fn color(value: &str) -> Color {
    let channel = |i: usize| {
        let hex = value.get(1 + i..3 + i).unwrap_or("ff");
        f32::from(u8::from_str_radix(hex, 16).expect("invalid color")) / 255.0
    };

    Color::new(channel(0), channel(2), channel(4), channel(6))
}

fn attribute<'a>(node: XmlNode<'a, '_>, name: &str) -> &'a str {
    node.attribute(name)
        .unwrap_or_else(|| panic!("<{}> is missing {name}", node.tag_name().name()))
}

fn number(node: XmlNode<'_, '_>, name: &str) -> f32 {
    parse_number(attribute(node, name))
}

fn number_or(node: XmlNode<'_, '_>, name: &str, default: f32) -> f32 {
    optional_number(node, name).unwrap_or(default)
}

fn optional_number(node: XmlNode<'_, '_>, name: &str) -> Option<f32> {
    node.attribute(name).map(parse_number)
}

fn numbers(value: &str) -> Vec<f32> {
    value
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|part| !part.is_empty())
        .map(parse_number)
        .collect()
}

fn parse_number(value: &str) -> f32 {
    value
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("expected a number, got \"{value}\""))
}
