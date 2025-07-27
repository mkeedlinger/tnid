use crate::name_encoding_0;
use crate::utils;

pub const fn compile_name_valid_check(name: &str) {
    if let utils::NAME_MIN..=utils::NAME_MAX = name.len() {
        if !name.is_ascii() {
            panic!("Id name must be ascii");
        }
    } else {
        panic!("Id name length must be within range")
    }

    let bytes = name.as_bytes();
    let mut i = 0;

    'check_loop: while i < bytes.len() {
        let mut j = 0;
        while j < name_encoding_0::CHAR_MAPPING.len() {
            if name_encoding_0::CHAR_MAPPING[j].1 == bytes[i] {
                i += 1;
                continue 'check_loop;
            }
            j += 1;
        }

        panic!("Invalid char in name");
    }
}

fn id_name_mask(name: &str) -> u128 {
    let mut mask = 0u128;

    debug_assert!(name.is_ascii());

    let name_bytes = name.as_bytes();

    debug_assert!(name_bytes.len() <= utils::NAME_MAX);

    for &name_char in name.as_bytes() {
        let encoding_mapping = name_encoding_0::CHAR_MAPPING
            .iter()
            .find(|(_encoded, from_char)| *from_char == name_char);

        let Some((encoded_byte, _)) = encoding_mapping else {
            panic!("ID name must be a-z0-4");
        };

        debug_assert!(*encoded_byte < 32);

        mask <<= name_encoding_0::CHAR_BIT_LENGTH;
        mask |= *encoded_byte as u128;
    }

    let needed_padding_chars = utils::NAME_MAX - name.len();
    for _ in 0..needed_padding_chars {
        mask <<= name_encoding_0::CHAR_BIT_LENGTH;
    }

    mask <<= 108;

    mask
}

fn years_mask(years_since_unix_epoch: u8) -> u128 {
    let mut mask = 0u128;

    let years_since_unix_epoch = years_since_unix_epoch as u128;

    mask |= years_since_unix_epoch;
    mask <<= 100;

    mask
}

fn year_seconds_mask(seconds_since_year_start: u32) -> u128 {
    debug_assert!(seconds_since_year_start.leading_zeros() >= 7);
    let seconds_since_year_start = seconds_since_year_start as u128;

    let mut mask = 0u128;

    mask |= seconds_since_year_start >> 5;

    mask <<= 9;
    mask |= seconds_since_year_start << 123 >> 123;

    mask <<= 71;

    mask
}

fn random_bits_mask(random: u128) -> u128 {
    const MASK: u128 = 0x00000000_0000_007f_0fff_ffffffffffff;

    random & MASK
}

pub fn make_from_parts(
    name: &str,
    years_since_unix_epoch: u8,
    seconds_since_year_start: u32,
    random: u128,
) -> u128 {
    let mut id = 0u128;

    id |= id_name_mask(name);

    id |= years_mask(years_since_unix_epoch);
    id |= year_seconds_mask(seconds_since_year_start);

    id |= utils::metadata_mask(0);

    id |= random_bits_mask(random);

    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_map_size() {
        assert_eq!(
            name_encoding_0::CHAR_MAPPING.len(),
            (2u8.pow(name_encoding_0::CHAR_BIT_LENGTH as u32) - 1) as usize
        );
    }

    #[test]
    fn name_mask_correct_location() {
        let mask = id_name_mask("zzzz");

        assert_eq!(mask.leading_zeros(), 0);
        assert_eq!(mask.leading_ones(), 20);

        assert_eq!(mask.trailing_zeros(), 108);
    }

    #[test]
    fn name_map_sorts() {
        let mut entries = name_encoding_0::CHAR_MAPPING.iter();
        let mut last = entries.next().unwrap();

        for next in entries {
            assert!(last.0 < next.0);
            assert!(last.1 < next.1);

            last = next;
        }
    }

    #[test]
    fn year_mask_correct_location() {
        let mask = years_mask(u8::MAX);

        assert_eq!(mask.leading_zeros(), 20);
        assert_eq!(mask.trailing_zeros(), 100);
        assert_eq!(mask.count_ones(), 8);
    }

    #[test]
    fn year_seconds_mask_correct_location() {
        let mask = year_seconds_mask(u32::MAX << 7 >> 7);

        assert_eq!(mask.leading_zeros(), 28);
        assert_eq!(mask.count_ones(), 25);
        assert_eq!(mask.trailing_zeros(), 71);
    }

    #[test]
    fn random_bits_mask_correct_location() {
        let mask = random_bits_mask(u128::MAX);

        dbg!(format!("{:0128b}", mask));

        assert_eq!(mask.leading_zeros(), 57);
        assert_eq!(mask.trailing_zeros(), 0);
        assert_eq!(mask.count_ones(), 67);
    }
}
