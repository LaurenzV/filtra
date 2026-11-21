use crate::{Source, Target};

/// `feOffset`, by whole pixels.
pub fn offset(src: Source<'_>, mut dst: Target<'_>, offset: [i64; 2]) {
    let [dx, dy] = offset;
    let x = i64::from(dst.rect().x0).saturating_sub(dx);

    for (y, row) in dst.rows_mut() {
        src.copy_row(i64::from(y).saturating_sub(dy), x, row);
    }
}
