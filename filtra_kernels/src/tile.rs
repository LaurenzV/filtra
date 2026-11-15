use crate::{IntRect, Source, Target};

/// Fills the target with copies of the `cell` part of `src`, one of them at the cell's position.
pub fn tile(src: Source<'_>, cell: IntRect, mut dst: Target<'_>) {
    let rect = dst.rect();

    if cell.is_empty() {
        for (_, row) in dst.rows_mut() {
            row.fill(0);
        }

        return;
    }

    let period = cell.width() as usize * 4;
    let first = wrap(rect.x0, cell.x0, cell.width());

    for (y, row) in dst.rows_mut().take(cell.height() as usize) {
        let source_y = wrap(y, cell.y0, cell.height());
        let filled = period.min(row.len());
        let head = (((i64::from(cell.x1) - i64::from(first)) * 4) as usize).min(filled);

        read(src, source_y, first, &mut row[..head]);
        read(src, source_y, cell.x0, &mut row[head..filled]);
        repeat(row, filled);
    }

    dst.repeat_rows(cell.height() as usize);
}

fn wrap(x: i32, start: i32, len: u32) -> i32 {
    (i64::from(start) + (i64::from(x) - i64::from(start)).rem_euclid(i64::from(len))) as i32
}

fn read(src: Source<'_>, y: i32, x0: i32, out: &mut [u8]) {
    let source = src.rect();
    let x1 = i64::from(x0) + (out.len() / 4) as i64;
    let from = i64::from(x0).max(i64::from(source.x0)).min(x1);
    let to = x1.min(i64::from(source.x1)).max(from);
    let (start, end) = (
        (from - i64::from(x0)) as usize * 4,
        (to - i64::from(x0)) as usize * 4,
    );

    out[..start].fill(0);

    match src.row(y, from as i32, to as i32) {
        Some(source_row) => out[start..end].copy_from_slice(source_row),
        None => out[start..end].fill(0),
    }

    out[end..].fill(0);
}

fn repeat(data: &mut [u8], mut filled: usize) {
    while filled < data.len() {
        let len = filled.min(data.len() - filled);
        data.copy_within(..len, filled);
        filled += len;
    }
}
