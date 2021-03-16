/// Converts a slice of f32's to u8's
pub fn slice_f32_to_u8(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

/// Converts a slice of f32's to u8's
pub fn slice_u32_to_u8(v: &[u32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

/// Converts a 1d index to 2d indexes
pub fn index_1d_to_2d(i: usize, row_size: usize) -> (usize, usize) {
    // https://softwareengineering.stackexchange.com/a/212813
    let x = i % row_size;
    let y = i / row_size;

    (x, y)
}

/// Converts a 2d index to a 1d index
pub fn index_2d_to_1d(x: usize, y: usize, row_size: usize) -> usize {
    // https://softwareengineering.stackexchange.com/a/212813
    x + row_size * y
}
