//! Applies SVG and CSS filter effects to pixmaps.
//!
//! A filter is described with the types in [`types`] in user space. [`apply`] plans it for the
//! pixels of the source with [`filtra_plan`] and executes the plan with the kernels of
//! [`filtra_kernels`] on premultiplied RGBA8 pixmaps.

mod cpu;

pub use filtra_kernels::{IntRect, Pixmap, PixmapMut};
pub use filtra_plan::Transform;
pub use filtra_types as types;

/// Applies `filter` to `pixmap`, whose pixels are placed in user space by `transform`, and
/// replaces its pixels with the result.
pub fn apply<I>(
    filter: &types::Filter<I>,
    pixmap: PixmapMut<'_>,
    transform: Transform,
    mut render_image: impl FnMut(&I, types::Rect, IntRect, &mut [u8]),
) {
    let bounds = pixmap.bounds();
    let bounds = filtra_plan::IntRect::new(bounds.x0, bounds.y0, bounds.x1, bounds.y1);
    let plan = filtra_plan::plan(filter, transform, bounds);
    let images: Vec<(&I, types::Rect)> = (filter.nodes().iter())
        .zip(filter.subregions())
        .filter_map(|(node, subregion)| match &node.primitive {
            types::Primitive::Image(image) => Some((image, subregion)),
            _ => None,
        })
        .collect();

    cpu::execute(&plan, pixmap, &mut |image, rect, data| {
        let (image, subregion) = images[image];
        render_image(image, subregion, rect, data);
    });
}
