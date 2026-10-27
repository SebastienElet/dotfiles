pub fn approximate_count(value: u64) -> f64 {
    let [
        high_first,
        high_second,
        high_third,
        high_fourth,
        low_first,
        low_second,
        low_third,
        low_fourth,
    ] = value.to_be_bytes();
    let high = f64::from(u32::from_be_bytes([
        high_first,
        high_second,
        high_third,
        high_fourth,
    ]));
    let low = f64::from(u32::from_be_bytes([
        low_first, low_second, low_third, low_fourth,
    ]));
    high.mul_add(4_294_967_296.0, low)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_counts_once_at_the_binary64_precision_boundary() {
        for (count, expected) in [
            (0, 0.0_f64),
            (1, 1.0),
            (u64::from(u32::MAX), 4_294_967_295.0),
            (1_u64 << 32, 4_294_967_296.0),
            ((1_u64 << 53) - 1, 9_007_199_254_740_991.0),
            (1_u64 << 53, 9_007_199_254_740_992.0),
            ((1_u64 << 53) + 1, 9_007_199_254_740_992.0),
            ((1_u64 << 53) + 3, 9_007_199_254_740_996.0),
            (u64::MAX, 2.0_f64.powi(64)),
        ] {
            assert_eq!(approximate_count(count).to_bits(), expected.to_bits());
        }
    }
}
