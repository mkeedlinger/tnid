#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]

// todo
// #![warn(missing_docs)]

use fpe::ff1::NumeralString;
use rand::random;
use std::marker::PhantomData;
use time::OffsetDateTime;

mod data_encoding;
mod error;
mod name_encoding;
mod utils;
mod uuidlike;
mod v0;
mod v1;

pub use error::Error;
pub use uuidlike::UUIDLike;

/// The 4 possible TNID variants
///
/// Similar to UUID variants, TNID variants have different construction that makes them useful for different situations.
#[derive(Debug, PartialEq)]
pub enum TNIDVariant {
    /// V0 is most like UUIDv7, and is meant to be time-sortable
    V0,
    /// V1 is most like UUIDv4, and is meant to maximize entropy (randomness)
    V1,
    /// V2 is undefined but reserved for future use
    V2,
    /// V3 is undefined but reserved for future use
    V3,
}

/// Intended to be used on empty structs to create type checked TNID names.
///
/// ```rust
/// # use tnid::TNIDName;
/// # use tnid::TNID;
///
/// struct ExampleName;
/// impl TNIDName for ExampleName {
///     const ID_NAME: &str = "ex";
/// }
///
/// # let _ = TNID::<ExampleName>::new_v0();
/// ```
///
/// The string you set as `ID_NAME` is checked to be a valid TNID name at compile time (as long as you actually use the )
/// ```rust,should_panic
/// # use tnid::TNIDName;
/// # use tnid::TNID;
///
/// struct InvalidName;
/// impl TNIDName for InvalidName {
///     const ID_NAME: &str = "2long";
/// }
///
/// # let _ = TNID::<InvalidName>::new_v0();
/// ```
pub trait TNIDName {
    const ID_NAME: &'static str;
    const NAME_IS_VALID: () = name_encoding::name_valid_check(Self::ID_NAME);
}

/// The base TNID type
///
/// Makes use of the [`TNIDName`] trait for static checking of the different names
///
/// In general, TNIDs try to be relatively strict about how they can be used and represented at compile time. That means that any given instance of a TNID *should* be valid. In cases where you want to work with or inspect potentially invalid TNIDs, use a [`UUIDLike`].
pub struct TNID<Name: TNIDName> {
    id_name: PhantomData<Name>,
    id: u128,
}

impl<Name: TNIDName> TNID<Name> {
    const NAME_IS_VALID: () = name_encoding::name_valid_check(Name::ID_NAME);
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
        Self::NAME_IS_VALID;

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

        let epoch_millis = (when.unix_timestamp_nanos() / 1000 / 1000) as u64;

        let random_bits: u64 = random();

        let id = v0::make_from_parts(id_name, epoch_millis, random_bits);

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
        todo!()
    }

    pub fn from_u128(num: u128) -> Result<Self, Error> {
        todo!()
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
            test_time += Duration::milliseconds(1);
            let id: TNID<TestId> = TNID::new_v0_at_time(test_time);

            assert!(last_id.as_u128() < id.as_u128());
            assert!(last_id.to_tnid_string() < id.to_tnid_string());

            last_id = id;
        }
    }

    #[test]
    fn tnid_variant_returns_v0() {
        let id: TNID<TestId> = TNID::new_v0();
        assert_eq!(id.tnid_variant(), TNIDVariant::V0);
    }
}
