pub const NAME_MIN: usize = 1;
pub const NAME_MAX: usize = 4;

pub const NAME_ENCODING_BIT_LENGTH: u8 = 5;
pub const NAME_ENCODING: phf::OrderedMap<u8, u8> = phf::phf_ordered_map! {
    1u8 => b'0',
    2u8 => b'1',
    3u8 => b'2',
    4u8 => b'3',
    5u8 => b'4',

    6u8 => b'a',
    7u8 => b'b',
    8u8 => b'c',
    9u8 => b'd',
    10u8 => b'e',
    11u8 => b'f',
    12u8 => b'g',
    13u8 => b'h',
    14u8 => b'i',
    15u8 => b'j',
    16u8 => b'k',
    17u8 => b'l',
    18u8 => b'm',
    19u8 => b'n',
    20u8 => b'o',
    21u8 => b'p',
    22u8 => b'q',
    23u8 => b'r',
    24u8 => b's',
    25u8 => b't',
    26u8 => b'u',
    27u8 => b'v',
    28u8 => b'w',
    29u8 => b'x',
    30u8 => b'y',
    31u8 => b'z',
};

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
