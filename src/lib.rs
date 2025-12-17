use fpe::ff1::NumeralString;
use rand::random;
use std::marker::PhantomData;
use time::{Date, OffsetDateTime, Time};

mod data_encoding;
mod error;
mod name_encoding;
mod utils;
mod v0;
mod v1;

pub use error::Error;

pub enum TNIDVariant {
    V0,
    V1,
    V2,
    V3,
}

pub trait TNIDName {
    const ID_NAME: &'static str;
    const NAME_IS_VALID: () = name_encoding::name_valid_check(Self::ID_NAME);
}

pub struct TNID<Name: TNIDName> {
    id_name: PhantomData<Name>,
    id: u128,
}

impl<Name: TNIDName> TNID<Name> {
    pub fn name(&self) -> &'static str {
        Name::ID_NAME
    }

    /// The TNID name when the
    /// bytes are encoded using hex, like they commonly are for UUIDs.
    pub fn name_hex(&self) -> String {
        let hex = format!("{:05x}", self.id >> 108);

        debug_assert_eq!(hex.len(), 5);

        hex
    }

    pub fn as_u128(&self) -> u128 {
        // put here since this is unlikely to be refactored
        #![allow(path_statements)] // access causes desired effect: panic on unsatisfied contraint at compile time
        Name::NAME_IS_VALID;

        self.id
    }

    /// Same as [`Self::new_v0`]
    pub fn new_time_ordered() -> Self {
        Self::new_v0()
    }

    /// Generates a new v0 TNID
    pub fn new_v0() -> Self {
        Self::new_v0_at_time(OffsetDateTime::now_utc())
    }

    pub fn new_high_entropy() -> Self {
        Self::new_v1()
    }

    /// Generates a new v1 TNID (high entropy)
    pub fn new_v1() -> Self {
        Self::new_v1_with_random(random())
    }

    /// Generates a new v1 TNID with specified randomness
    fn new_v1_with_random(random_bits: u128) -> Self {
        let id_name = Name::ID_NAME;
        debug_assert!(id_name.len() <= name_encoding::NAME_MAX_CHARS);

        let id = v1::make_from_parts(id_name, random_bits);

        Self {
            id_name: PhantomData,
            id,
        }
    }

    fn new_v0_at_time(when: OffsetDateTime) -> Self {
        let id_name = Name::ID_NAME;
        debug_assert!(id_name.len() <= name_encoding::NAME_MAX_CHARS);

        let years_since_unix_epoch = {
            let years_since_unix_epoch = when.year() - 2020 as i32;

            // Using a cast instead of .try_into() means that
            // it won't fail if by some craziness it is far in the future
            years_since_unix_epoch as u8
        };

        let seconds_since_year_start = {
            let beginning_of_year = OffsetDateTime::new_utc(
                Date::from_calendar_date(when.year(), time::Month::January, 1).unwrap(),
                Time::MIDNIGHT,
            );

            // Using a cast to avoid an unwrap, negatives will be 0
            let seconds_since_year_start: u32 = (when - beginning_of_year).whole_seconds() as u32;

            debug_assert!(2usize.pow(26) > seconds_since_year_start as usize);

            seconds_since_year_start
        };

        let random_bits: u128 = random();

        let id = v0::make_from_parts(
            id_name,
            years_since_unix_epoch,
            seconds_since_year_start,
            random_bits,
        );

        Self {
            id_name: PhantomData,
            id,
        }
    }

    pub fn to_tnid_string(&self) -> String {
        let mut stripped_id_info = 0u128;

        // get up to UUID version
        stripped_id_info |= self.id << 20 >> 20 >> 80;

        // get up to variant info
        stripped_id_info <<= 12;
        stripped_id_info |= self.id << 52 >> 52 >> 64;

        // get TNI variant and rest of random
        stripped_id_info <<= 62;
        stripped_id_info |= self.id << 66 >> 66;

        // 26 most significant bits should be zeros
        debug_assert!(stripped_id_info < u128::MAX << 22 >> 22);

        let rest = base58ck::encode(&stripped_id_info.to_be_bytes()[4..]);

        debug_assert!(rest.len() <= 17);

        format!("{}.{:1>17}", self.name(), rest)
    }

    pub fn tnid_variant(&self) -> TNIDVariant {
        let variant_bits = (self.id >> 60) & 0b11;

        match variant_bits {
            0 => TNIDVariant::V0,
            1 => TNIDVariant::V1,
            2 => TNIDVariant::V2,
            3 => TNIDVariant::V3,
            _ => unreachable!("2 bits can only have 4 values"),
        }
    }

    /// Convert to UUID hex string format (lowercase)
    pub fn to_uuid_string(&self) -> String {
        self.to_uuid_string_cased(false)
    }

    /// Convert to UUID hex string format with specified case
    pub fn to_uuid_string_cased(&self, uppercase: bool) -> String {
        let id = self.id;

        // Format as UUID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let first_section = (id >> 96) as u32;
        let second_section = ((id >> 80) & 0xFFFF) as u16;
        let third_section = ((id >> 64) & 0xFFFF) as u16;
        let fourth_section = ((id >> 48) & 0xFFFF) as u16;
        let fifth_section = (id & 0xFFFFFFFFFFFF) as u64;
        if uppercase {
            format!(
                "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
                first_section, second_section, third_section, fourth_section, fifth_section
            )
        } else {
            format!(
                "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                first_section, second_section, third_section, fourth_section, fifth_section
            )
        }
    }

    /// Parse a UUID hex string into a TNID
    pub fn parse_uuid_string(uuid_string: &str) -> Result<Self, Error> {
        if uuid_string.len() != 36 {
            return Err(Error::InvalidUuidFormat);
        }

        if uuid_string.as_bytes()[8] != b'-'
            || uuid_string.as_bytes()[13] != b'-'
            || uuid_string.as_bytes()[18] != b'-'
            || uuid_string.as_bytes()[23] != b'-'
        {
            return Err(Error::InvalidUuidFormat);
        }

        let section1 =
            u32::from_str_radix(&uuid_string[0..8], 16).map_err(|_| Error::InvalidUuidFormat)?;
        let section2 =
            u16::from_str_radix(&uuid_string[9..13], 16).map_err(|_| Error::InvalidUuidFormat)?;
        let section3 =
            u16::from_str_radix(&uuid_string[14..18], 16).map_err(|_| Error::InvalidUuidFormat)?;
        let section4 =
            u16::from_str_radix(&uuid_string[19..23], 16).map_err(|_| Error::InvalidUuidFormat)?;
        let section5 =
            u64::from_str_radix(&uuid_string[24..36], 16).map_err(|_| Error::InvalidUuidFormat)?;

        // Reconstruct u128 from sections
        let id = ((section1 as u128) << 96)
            | ((section2 as u128) << 80)
            | ((section3 as u128) << 64)
            | ((section4 as u128) << 48)
            | (section5 as u128);

        Ok(Self {
            id_name: PhantomData,
            id,
        })
    }

    /// Create a TNID from a u128, validating it's a proper TNID format
    pub fn from_u128(num: u128) -> Result<Self, Error> {
        // Check UUID version bits (74-71) should be 1000 (8)
        let version_bits = (num >> 71) & 0b1111;
        if version_bits != 8 {
            return Err(Error::InvalidUuidVersion);
        }

        // Check UUID variant bits (70-69) should be 10
        let variant_bits = (num >> 69) & 0b11;
        if variant_bits != 0b10 {
            return Err(Error::InvalidUuidVersion);
        }

        // Check TNID variant bits (68-67) should be valid (0-3)
        let tnid_variant_bits = (num >> 67) & 0b11;
        if tnid_variant_bits > 3 {
            return Err(Error::InvalidUuidVersion);
        }

        // Extract and validate name section (bits 127-108, 20 bits total)
        let name_bits = num >> 108;

        // Validate each 5-bit character encoding
        for i in 0..4 {
            let char_encoding = (name_bits >> (15 - i * 5)) & 0b11111;

            // Check if this encoding exists in our character mapping
            // 0 is valid (null terminator for padding), 1-31 should map to valid chars
            if char_encoding > 31 {
                return Err(Error::InvalidNameEncoding);
            }

            // If not null terminator, verify it maps to a valid character
            if char_encoding != 0 {
                let has_mapping = name_encoding::CHAR_MAPPING
                    .iter()
                    .any(|(encoded, _)| *encoded == char_encoding as u8);

                if !has_mapping {
                    return Err(Error::InvalidNameEncoding);
                }
            }
        }

        Ok(Self {
            id_name: PhantomData,
            id: num,
        })
    }

    pub fn encrypt_v0_to_v1(&self, secret: &[u8]) -> Result<Self, ()> {
        use aes::Aes128;
        use fpe::ff1::{BinaryNumeralString, FF1};

        let data_bits = data_encoding::extract_data_bits(self.id).to_le_bytes();

        let ff = FF1::<Aes128>::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 0], 2).unwrap();

        let encrypted = ff
            .encrypt(&[], &BinaryNumeralString::from_bytes_le(&data_bits))
            .unwrap()
            .to_bytes_le();

        let encrypted = encrypted.as_slice();

        dbg!(encrypted);

        let encrypted: [u8; 16] = encrypted.try_into().unwrap();

        let encrypted = u128::from_le_bytes(encrypted);

        dbg!(data_encoding::extract_data_bits(self.id), encrypted);

        Ok(Self {
            id_name: PhantomData,
            id: encrypted,
        })
    }
}

impl<Name: TNIDName> std::fmt::Display for TNID<Name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_tnid_string())
    }
}

impl<Name: TNIDName> std::fmt::Debug for TNID<Name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_tnid_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fpe_test() {
        let id: TNID<TestId> = TNID::new_time_ordered();
        id.encrypt_v0_to_v1(&[1, 2]).unwrap();
    }

    struct TestId;
    impl TNIDName for TestId {
        const ID_NAME: &'static str = "test";
    }

    #[test]
    fn variant0_is_k_sortable() {
        use time::Duration;

        let mut test_time = OffsetDateTime::now_utc();
        let mut last_id: TNID<TestId> = TNID::new_v0_at_time(test_time);

        for _ in 1..10_000 {
            test_time += Duration::milliseconds(1001);
            let id: TNID<TestId> = TNID::new_v0_at_time(test_time);

            assert!(last_id.as_u128() < id.as_u128());
            assert!(last_id.to_tnid_string() < id.to_tnid_string());

            last_id = id;
        }
    }

    #[test]
    fn variant0_makes_correctly() {
        let name = "test";
        let years_since_unix_epoch = 0x42;
        let seconds_since_year_start = 0b00000001_00011000_00011000_00110001;
        let random = 0x00000000_0000_0071_0234_56789abcdeff;

        let output = v0::make_from_parts(
            name,
            years_since_unix_epoch,
            seconds_since_year_start,
            random,
        );

        let name_section = 0b11001_01010_11000_11001u128 << 108;
        let years_section = 0x42u128 << 100;
        let year_seconds_section = 0b10001100000011000001u128 << 80;
        let year_seconds_section_2 = 0b10001u128 << 71;
        let meta_section = 0x00000000_0000_8000_8000_000000000000;

        assert_eq!(
            output,
            name_section
                | years_section
                | year_seconds_section
                | year_seconds_section_2
                | meta_section
                | random
        );
    }

    #[test]
    fn tnid_variant_returns_v0() {
        let id: TNID<TestId> = TNID::new_v0();
        assert!(matches!(id.tnid_variant(), TNIDVariant::V0));
    }

    #[test]
    fn from_u128_validation() {
        // Valid TNID should work
        let valid_tnid: TNID<TestId> = TNID::new_v0();
        assert!(TNID::<TestId>::from_u128(valid_tnid.as_u128()).is_ok());

        // Invalid UUID version (not version 8)
        let invalid_version = 0x00000000_0000_7000_8000_000000000000u128; // version 7
        assert!(matches!(
            TNID::<TestId>::from_u128(invalid_version),
            Err(Error::InvalidUuidVersion)
        ));

        // Invalid UUID variant (not variant 2)
        let invalid_variant = 0x00000000_0000_8000_4000_000000000000u128; // variant 1
        assert!(matches!(
            TNID::<TestId>::from_u128(invalid_variant),
            Err(Error::InvalidUuidVersion)
        ));

        // Invalid name encoding (character 32 doesn't exist)
        let invalid_name = (32u128 << 123) | 0x00000000_0000_8000_8000_000000000000u128;
        assert!(matches!(
            TNID::<TestId>::from_u128(invalid_name),
            Err(Error::InvalidNameEncoding)
        ));
    }
}
