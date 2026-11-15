use crate::{Source, Target};

/// Copies `src` moved by `dx` and `dy` whole pixels.
pub fn offset(src: Source<'_>, mut dst: Target<'_>, dx: i64, dy: i64) {
    let (rect, source) = (dst.rect(), src.rect());
    let x0 = i64::from(rect.x0).max(i64::from(source.x0).saturating_add(dx));
    let x1 = i64::from(rect.x1).min(i64::from(source.x1).saturating_add(dx));

    for (y, row) in dst.rows_mut() {
        let source_y = i64::from(y).saturating_sub(dy);
        let source_row = (x0 < x1)
            .then(|| i32::try_from(source_y).ok())
            .flatten()
            .and_then(|source_y| src.row(source_y, (x0 - dx) as i32, (x1 - dx) as i32));

        let Some(source_row) = source_row else {
            row.fill(0);
            continue;
        };

        let start = (x0 - i64::from(rect.x0)) as usize * 4;
        let end = start + source_row.len();

        row[..start].fill(0);
        row[start..end].copy_from_slice(source_row);
        row[end..].fill(0);
    }
}
