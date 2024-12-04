use crate::utils;

pub const fn compile_name_valid_check(name: &str) {
    if let utils::NAME_MIN..=utils::NAME_MAX = name.len() {
        if !name.is_ascii() {
            panic!("Id name must be ascii");
        }
    } else {
        panic!("Id name length must be between within range")
    }
}

fn id_name_mask(name: &str) -> u128 {
    let mut mask = 0u128;

    debug_assert!(name.is_ascii());

    let name_bytes = name.as_bytes();

    debug_assert!(name_bytes.len() <= utils::NAME_MAX);

    for &name_char in name.as_bytes() {
        let encoding_mapping = utils::NAME_ENCODING
            .entries()
            .find(|(_encoded, &from_char)| from_char == name_char);

        let Some((&encoded_byte, _)) = encoding_mapping else {
            panic!("ID name must be a-z0-4");
        };

        debug_assert!(encoded_byte < 32);

        mask <<= utils::NAME_ENCODING_BIT_LENGTH;
        mask |= encoded_byte as u128;
    }

    let needed_padding_chars = utils::NAME_MAX - name.len();
    for _ in 0..needed_padding_chars {
        mask <<= utils::NAME_ENCODING_BIT_LENGTH;
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
            utils::NAME_ENCODING.len(),
            (2u8.pow(utils::NAME_ENCODING_BIT_LENGTH as u32) - 1) as usize
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
        let mut entries = utils::NAME_ENCODING.entries();
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
