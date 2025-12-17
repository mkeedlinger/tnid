use crate::name_encoding;
use crate::utils;

fn random_bits_mask(random: u128) -> u128 {
    todo!()
}

pub fn make_from_parts(name: &str, random: u128) -> u128 {
    let mut id = 0u128;

    id |= name_encoding::id_name_mask(name).unwrap();
    id |= utils::uuid_and_variant_mask(1);
    id |= random_bits_mask(random);

    id
}
