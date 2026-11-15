use std::sync::OnceLock;

use crate::pixmap::segments;
use crate::{Source, Target};

/// The color space of a pixmap's color values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// Gamma-encoded sRGB.
    Srgb,
    /// Linear-light sRGB.
    LinearRgb,
}

/// Converts the colors of `src` from one color space to another.
pub fn convert(src: Source<'_>, mut dst: Target<'_>, from: ColorSpace, to: ColorSpace) {
    let rect = dst.rect();
    let table = (from != to).then(|| table(to == ColorSpace::LinearRgb));

    for (y, row) in dst.rows_mut() {
        for (x0, x1) in segments(rect.x0, rect.x1, [src.rect()].into_iter()) {
            let output = &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4];

            match (src.row(y, x0, x1), table) {
                (None, _) => output.fill(0),
                (Some(source), None) => output.copy_from_slice(source),
                (Some(source), Some(table)) => {
                    let output = output.as_chunks_mut::<4>().0;

                    for (output, source) in output.iter_mut().zip(source.as_chunks::<4>().0) {
                        let results = &table[usize::from(source[3])];
                        *output = [
                            results[usize::from(source[0])],
                            results[usize::from(source[1])],
                            results[usize::from(source[2])],
                            source[3],
                        ];
                    }
                }
            }
        }
    }
}

// One exact result for every combination of alpha (outer index) and premultiplied color value.
fn table(to_linear: bool) -> &'static [[u8; 256]; 256] {
    static TO_LINEAR: OnceLock<Box<[[u8; 256]; 256]>> = OnceLock::new();
    static TO_SRGB: OnceLock<Box<[[u8; 256]; 256]>> = OnceLock::new();

    let build = move || {
        let mut table = Box::new([[0; 256]; 256]);

        for (alpha, row) in table.iter_mut().enumerate().skip(1) {
            let (unclamped, clamped) = row.split_at_mut(alpha);
            let alpha = alpha as f32;

            for (color, value) in unclamped.iter_mut().enumerate() {
                let color = color as f32 / alpha;
                let converted = if to_linear {
                    srgb_to_linear(color)
                } else {
                    linear_to_srgb(color)
                };

                *value = (converted * alpha + 0.5) as u8;
            }

            clamped.fill(alpha as u8);
        }

        table
    };

    if to_linear {
        TO_LINEAR.get_or_init(build)
    } else {
        TO_SRGB.get_or_init(build)
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
