use crate::Target;

/// Fills the target with a premultiplied color.
pub fn flood(mut dst: Target<'_>, color: [u8; 4]) {
    for (_, row) in dst.rows_mut() {
        row.as_chunks_mut::<4>().0.fill(color);
    }
}
