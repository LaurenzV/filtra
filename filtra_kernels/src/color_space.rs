use std::sync::OnceLock;

use crate::Target;

/// The color space of a pixmap's color values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// Gamma-encoded sRGB.
    Srgb,
    /// Linear-light sRGB.
    LinearRgb,
}

/// Converts the colors of the target from one color space to another.
pub fn convert_color_space(mut dst: Target<'_>, from: ColorSpace, to: ColorSpace) {
    if from == to {
        return;
    }

    let table = table(to == ColorSpace::LinearRgb);

    for (_, row) in dst.rows_mut() {
        for pixel in row.as_chunks_mut::<4>().0 {
            let results = &table[usize::from(pixel[3])];
            pixel[0] = results[usize::from(pixel[0])];
            pixel[1] = results[usize::from(pixel[1])];
            pixel[2] = results[usize::from(pixel[2])];
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
