use crate::{IntRect, Source, Target};

/// `feTile`.
pub fn tile(src: Source<'_>, mut dst: Target<'_>, cell: IntRect) {
    let rect = dst.rect();

    if cell.is_empty() {
        return crate::flood(dst, [0; 4]);
    }

    let period = (cell.width() as usize).saturating_mul(4);
    let first = wrap(rect.x0, cell.x0, cell.width());

    for (y, row) in dst.rows_mut().take(cell.height() as usize) {
        let source_y = wrap(y, cell.y0, cell.height());
        let filled = period.min(row.len());
        let head = ((i64::from(cell.x1) - i64::from(first)) * 4).min(filled as i64) as usize;

        src.copy_row(source_y.into(), first.into(), &mut row[..head]);
        src.copy_row(source_y.into(), cell.x0.into(), &mut row[head..filled]);
        repeat(row, filled);
    }

    dst.repeat_rows(cell.height() as usize);
}

fn wrap(x: i32, start: i32, len: u32) -> i32 {
    (i64::from(start) + (i64::from(x) - i64::from(start)).rem_euclid(i64::from(len))) as i32
}

fn repeat(data: &mut [u8], mut filled: usize) {
    while filled < data.len() {
        let len = filled.min(data.len() - filled);
        data.copy_within(..len, filled);
        filled += len;
    }
}
