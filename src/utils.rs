pub const NAME_MIN: usize = 1;
pub const NAME_MAX: usize = 4;

pub fn metadata_mask(variant: u8) -> u128 {
    const UUID_V8_MASK: u128 = 0x00000000_0000_8000_8000_000000000000;

    // should only take at most 2 bits
    debug_assert!(variant.leading_zeros() >= 6);

    let variant = variant as u128;

    UUID_V8_MASK | variant << 60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_mask_correct_location() {
        let mask = metadata_mask(0);

        assert_eq!(mask.leading_zeros(), 48);
        assert_eq!(mask.trailing_zeros(), 63);
        assert_eq!(mask.count_ones(), 2);
    }
}
