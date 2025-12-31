use aes::Aes128;
use fpe::ff1::{FF1, FlexibleNumeralString};

const RIGHT_SECRET_DATA_SECTION_MASK: u128 = 0x00000000_0000_0000_0fff_ffffffffffff;
const MIDDLE_SECRET_DATA_SECTION_MASK: u128 = 0x00000000_0000_0fff_0000_000000000000;
const LEFT_SECRET_DATA_SECTION_MASK: u128 = 0x00000fff_ffff_0000_0000_000000000000;

pub const COMPLETE_SECRET_DATA_MASK: u128 = RIGHT_SECRET_DATA_SECTION_MASK
    | MIDDLE_SECRET_DATA_SECTION_MASK
    | LEFT_SECRET_DATA_SECTION_MASK;

/// Extract secret data bits (excludes name, UUID version/variant, and TNID variant)
pub fn extract_secret_data_bits(id: u128) -> u128 {
    let extracted = id & RIGHT_SECRET_DATA_SECTION_MASK;

    const BETWEEN_MIDDLE_RIGHT: i32 = 4;
    let extracted = extracted | ((id & MIDDLE_SECRET_DATA_SECTION_MASK) >> BETWEEN_MIDDLE_RIGHT);

    const BETWEEN_LEFT_MIDDLE: i32 = BETWEEN_MIDDLE_RIGHT + 4;
    let extracted = extracted | ((id & LEFT_SECRET_DATA_SECTION_MASK) >> BETWEEN_LEFT_MIDDLE);

    extracted
}

/// Expand compacted secret data bits back into their positions (inverse of extract_secret_data_bits)
pub fn expand_secret_data_bits(bits: u128) -> u128 {
    // Right section stays in place
    let expanded = bits & RIGHT_SECRET_DATA_SECTION_MASK;

    // Middle section shifts left
    const BETWEEN_MIDDLE_RIGHT: i32 = 4;
    let middle_mask = MIDDLE_SECRET_DATA_SECTION_MASK >> BETWEEN_MIDDLE_RIGHT;
    let expanded = expanded | ((bits & middle_mask) << BETWEEN_MIDDLE_RIGHT);

    // Left section shifts left
    const BETWEEN_LEFT_MIDDLE: i32 = BETWEEN_MIDDLE_RIGHT + 4;
    let left_mask = LEFT_SECRET_DATA_SECTION_MASK >> BETWEEN_LEFT_MIDDLE;
    let expanded = expanded | ((bits & left_mask) << BETWEEN_LEFT_MIDDLE);

    expanded
}

const SECRET_DATA_BIT_NUM: u8 = COMPLETE_SECRET_DATA_MASK.count_ones() as u8;
const HEX_DIGIT_COUNT: usize = 25; // 100 bits / 4 bits per hex digit = 25

fn u128_to_hex_digits(data: u128) -> Vec<u16> {
    let mut hex_digits = Vec::with_capacity(HEX_DIGIT_COUNT);
    for i in 0..HEX_DIGIT_COUNT {
        let shift = (HEX_DIGIT_COUNT - 1 - i) * 4;
        hex_digits.push(((data >> shift) & 0xF) as u16);
    }
    hex_digits
}

fn hex_digits_to_u128(digits: Vec<u16>) -> u128 {
    let mut result = 0u128;
    for digit in digits {
        result = (result << 4) | (digit as u128);
    }
    result
}

pub fn encrypt(id_secret_data: u128, secret: &[u8; 16]) -> u128 {
    // Mask to only encrypt the lower 100 bits
    let mask = (1u128 << SECRET_DATA_BIT_NUM) - 1;
    let data = id_secret_data & mask;

    let hex_digits = u128_to_hex_digits(data);
    let numeral_string = FlexibleNumeralString::from(hex_digits);
    let ff1 = FF1::<Aes128>::new(secret, 16).unwrap(); // radix 16 for hex

    let encrypted = ff1.encrypt(&[], &numeral_string).unwrap();

    hex_digits_to_u128(encrypted.into())
}

pub fn decrypt(id_secret_data: u128, secret: &[u8; 16]) -> u128 {
    // Mask to only decrypt the lower 100 bits
    let mask = (1u128 << SECRET_DATA_BIT_NUM) - 1;
    let data = id_secret_data & mask;

    let hex_digits = u128_to_hex_digits(data);
    let numeral_string = FlexibleNumeralString::from(hex_digits);
    let ff1 = FF1::<Aes128>::new(secret, 16).unwrap(); // radix 16 for hex

    let decrypted = ff1.decrypt(&[], &numeral_string).unwrap();

    hex_digits_to_u128(decrypted.into())
}

#[cfg(test)]
mod tests {
    use std::u128;

    use super::*;

    #[test]
    fn secret_data_extract_correctly() {
        let extract = extract_secret_data_bits(u128::MAX);
        assert_eq!(extract.leading_zeros(), 28);
        assert_eq!(extract.count_ones(), SECRET_DATA_BIT_NUM.into());

        assert_eq!(
            COMPLETE_SECRET_DATA_MASK.count_ones(),
            SECRET_DATA_BIT_NUM.into()
        );

        let extract = extract_secret_data_bits(COMPLETE_SECRET_DATA_MASK);
        assert_eq!(extract.leading_zeros(), 28);
        assert_eq!(extract.count_ones(), SECRET_DATA_BIT_NUM.into());
    }

    #[test]
    fn secret_data_expand_correctly() {
        // Expand should produce the mask when given all 100 bits set
        let expanded = expand_secret_data_bits(u128::MAX);
        assert_eq!(expanded, COMPLETE_SECRET_DATA_MASK);
        assert_eq!(expanded.count_ones(), SECRET_DATA_BIT_NUM.into());
    }

    // todo: review, ai generated
    #[test]
    fn secret_data_roundtrip() {
        // Extract then expand should give back the original (masked)
        let original = COMPLETE_SECRET_DATA_MASK;
        let extracted = extract_secret_data_bits(original);
        let expanded = expand_secret_data_bits(extracted);
        assert_eq!(expanded, original);

        // Test with arbitrary pattern
        let pattern = 0x00000aaa_aaaa_0000_0555_555555555555u128;
        let extracted = extract_secret_data_bits(pattern);
        let expanded = expand_secret_data_bits(extracted);
        assert_eq!(
            expanded & COMPLETE_SECRET_DATA_MASK,
            pattern & COMPLETE_SECRET_DATA_MASK
        );
    }

    #[test]
    fn encryption_round_trip() {
        let secret = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let id_secret_data = extract_secret_data_bits(u128::MAX);
        let encrypted = encrypt(id_secret_data, &secret);

        let decrypted = decrypt(encrypted, &secret);

        dbg!(id_secret_data, encrypted, decrypted);

        assert_eq!(decrypted, id_secret_data);
    }
}
