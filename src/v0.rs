use crate::name_encoding;
use crate::utils;

fn millis_mask(millis_since_epoch: u64) -> u128 {
    let mut mask = 0u128;

    const FIRST_28_MASK: u64 = 0x0000_07ff_ffff_8000;
    mask |=
        ((millis_since_epoch & FIRST_28_MASK) as u128) << (FIRST_28_MASK.leading_zeros() + 64 - 20);

    const SECOND_12_MASK: u64 = 0x0000_0000_0000_7ff8;
    mask |= ((millis_since_epoch & SECOND_12_MASK) as u128)
        << (SECOND_12_MASK.leading_zeros() + 64 - 52);

    const LAST_3_MASK: u64 = 0x0000_0000_0000_0007;
    mask |= ((millis_since_epoch & LAST_3_MASK) as u128) << (LAST_3_MASK.leading_zeros() + 64 - 68);

    mask
}

fn random_bits_mask(random: u64) -> u128 {
    const MASK: u128 = 0x00000000_0000_0000_01ff_ffffffffffff;

    let random = random as u128;

    random & MASK
}

pub fn make_from_parts(name: &str, epoch_millis: u64, random: u64) -> u128 {
    let mut id = 0u128;

    id |= name_encoding::id_name_mask(name).unwrap(); // change unwrap to handle errors

    id |= millis_mask(epoch_millis);

    id |= utils::uuid_and_variant_mask(0);

    id |= random_bits_mask(random);

    id
}

#[cfg(test)]
mod tests {
    use std::u64;

    use super::*;

    #[test]
    fn name_map_size() {
        assert_eq!(
            name_encoding::CHAR_MAPPING.len(),
            (2u8.pow(name_encoding::CHAR_BIT_LENGTH as u32) - 1) as usize
        );
    }

    #[test]
    fn name_mask_correct_location() {
        let mask = name_encoding::id_name_mask("zzzz").unwrap();

        assert_eq!(mask.leading_zeros(), 0);
        assert_eq!(mask.leading_ones(), 20);

        assert_eq!(mask.trailing_zeros(), 108);
    }

    #[test]
    fn name_map_sorts() {
        let mut entries = name_encoding::CHAR_MAPPING.iter();
        let mut last = entries.next().unwrap();

        for next in entries {
            assert!(last.0 < next.0);
            assert!(last.1 < next.1);

            last = next;
        }
    }

    #[test]
    fn millis_mask_correct_location() {
        let mask = millis_mask(u64::MAX);

        assert_eq!(mask.leading_zeros(), 20);
        assert_eq!(mask.count_ones(), 43);
        assert_eq!(mask.trailing_zeros(), 57);
    }

    #[test]
    fn random_bits_mask_correct_location() {
        let mask = random_bits_mask(u64::MAX);

        const FRONT_ZEROS: u32 = 71;
        const BACK_ONES: u32 = 57;

        assert_eq!(BACK_ONES + FRONT_ZEROS, 128);
        assert_eq!(mask.leading_zeros(), FRONT_ZEROS);
        assert_eq!(mask.count_ones(), BACK_ONES);
    }
}
