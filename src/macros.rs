#[macro_export]
macro_rules! parse_number {
    ($coordinate: expr) => ({
        use std::str;
        use std::ops::Not;
        let index: usize = $coordinate.iter().position(|c|
            (b'0'..b':').contains(c).not()
        ).unwrap_or(
            $coordinate.len()
        );
        let term: &[u8] = &$coordinate[..index];
        if let Ok(number) = unsafe {
            usize::from_str_radix(str::from_utf8_unchecked(term), 10)
        } {
            let next: &[u8] = &$coordinate[index..];
            Some((number, next))
        }
        else {
            None
        }
    });
}
