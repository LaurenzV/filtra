//! Visual tests for filtra.

#![cfg(test)]

mod blend;
mod color_matrix;
mod component_transfer;
mod composite;
mod convolve_matrix;
mod diffuse_lighting;
mod displacement_map;
mod drop_shadow;
mod filter;
mod flood;
mod functions;
mod gaussian_blur;
mod image;
mod linear_rgb;
mod merge;
mod morphology;
mod offset;
mod region;
mod specular_lighting;
mod svg;
mod tile;
mod transform;
mod turbulence;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use filtra::Transform;
use vello_cpu::color::palette::css::WHITE;
use vello_cpu::kurbo::{Affine, Rect};
use vello_cpu::peniko::color::AlphaColor;
use vello_cpu::peniko::{ImageQuality, ImageSampler};
use vello_cpu::{
    Image, ImageSource, PixelMetadata, Pixmap, RasterizerSettings, RenderContext, Resources,
    TargetInit,
};

use svg::{Document, Shape};

/// Set to write the results as the new references instead of comparing with them.
const REPLACE: &str = "FILTRA_REPLACE";

/// Checks the test `name`, for example `flood/subregion`, against its reference.
fn check(name: &str) {
    check_at(name, 1);
}

/// Checks the test `name`, rendered at `scale` times its size, against its reference.
fn check_at(name: &str, scale: u32) {
    let text = std::fs::read_to_string(path(&format!("svg/{name}.svg"))).unwrap();
    let actual = on_white(render(&svg::parse(&text), scale));
    let reference = path(&format!("references/{name}.png"));

    if std::env::var_os(REPLACE).is_some() {
        std::fs::create_dir_all(reference.parent().unwrap()).unwrap();
        std::fs::write(&reference, actual.into_png().unwrap()).unwrap();

        return;
    }

    let expected = std::fs::File::open(&reference)
        .ok()
        .and_then(|file| Pixmap::from_png(std::io::BufReader::new(file)).ok());
    let matches = expected.is_some_and(|expected| {
        (expected.width(), expected.height()) == (actual.width(), actual.height())
            && expected.data_as_u8_slice() == actual.data_as_u8_slice()
    });

    if !matches {
        let result = path(&format!(
            "../target/filtra-diffs/{}.png",
            name.replace('/', "_")
        ));
        std::fs::create_dir_all(result.parent().unwrap()).unwrap();
        std::fs::write(&result, actual.into_png().unwrap()).unwrap();

        panic!(
            "{name} differs from {}; got {} ({REPLACE}=1 replaces the reference)",
            reference.display(),
            result.display()
        );
    }
}

fn render(document: &Document, scale: u32) -> filtra::Pixmap {
    let (width, height) = (document.width * scale, document.height * scale);
    let scale_transform = Affine::scale(f64::from(scale));
    let source = filtra::Pixmap::from_data(
        rasterize(&document.shapes, width, height, scale_transform),
        width,
        height,
    );

    match &document.filter {
        Some(filter) => {
            let transform = Transform::new(scale as f32, scale as f32, 0.0, 0.0);
            filtra::apply(
                filter,
                &source,
                transform,
                |shapes, subregion, rect, data| {
                    // Like `<use>`, a referenced element is placed at the subregion's origin.
                    let placement = Affine::translate((-f64::from(rect.x0), -f64::from(rect.y0)))
                        * scale_transform
                        * Affine::translate((f64::from(subregion.x), f64::from(subregion.y)));
                    let image = rasterize(shapes, rect.width(), rect.height(), placement);

                    // Drawing over `data` instead of replacing it shows if it isn't transparent.
                    draw_over(data, &image);
                },
            )
        }
        None => source,
    }
}

fn draw_over(data: &mut [u8], image: &[u8]) {
    for (pixel, color) in data
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(image.as_chunks::<4>().0)
    {
        let inverse = 255 - u16::from(color[3]);

        for (value, &source) in pixel.iter_mut().zip(color) {
            *value = (u16::from(source) + u16::from(*value) * inverse / 255) as u8;
        }
    }
}

fn rasterize(shapes: &[Shape], width: u32, height: u32, transform: Affine) -> Vec<u8> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let mut context = RenderContext::new(width as u16, height as u16);
    context.set_transform(transform);

    for shape in shapes {
        let color = shape.color;
        let [r, g, b, a] = [color.r, color.g, color.b, color.a].map(|c| (c * 255.0).round() as u8);
        context.set_paint(AlphaColor::from_rgba8(r, g, b, a));
        context.fill_path(&shape.path);
    }

    let mut target = Pixmap::new(width as u16, height as u16);
    context.flush();
    context.render(&mut target, &mut Resources::new());

    target.data_as_u8_slice().to_vec()
}

fn on_white(result: filtra::Pixmap) -> Pixmap {
    let (width, height) = (result.width() as u16, result.height() as u16);
    let data = result.into_data();
    let image = Pixmap::from_parts(data, width, height, PixelMetadata::default());

    let mut context = RenderContext::new(width, height);
    context.set_paint(Image {
        image: ImageSource::Pixmap(Arc::new(image)),
        sampler: ImageSampler {
            quality: ImageQuality::Low,
            ..ImageSampler::default()
        },
    });
    context.fill_rect(&Rect::new(0.0, 0.0, f64::from(width), f64::from(height)));
    context.flush();

    let mut target = Pixmap::new(width, height);
    let settings = RasterizerSettings {
        target_init: TargetInit::Clear(WHITE),
        ..RasterizerSettings::default()
    };
    context.render_with(&mut target, &mut Resources::new(), settings);

    target
}

fn path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
