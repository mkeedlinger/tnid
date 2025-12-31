#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]

// todo
// #![warn(missing_docs)]

use rand::random;
use std::marker::PhantomData;
use time::OffsetDateTime;

mod data_encoding;
#[cfg(feature = "encryption")]
mod encryption;
mod error;
mod name_encoding;
mod tnid_variant;
mod utils;
mod uuidlike;
mod v0;
mod v1;

pub use error::Error;
pub use tnid_variant::TNIDVariant;
pub use uuidlike::UUIDLike;

/// Intended to be used on empty structs to create type checked TNID names.
///
/// ```rust
/// # use tnid::TNIDName;
/// # use tnid::TNID;
///
/// struct ExampleName;
/// impl TNIDName for ExampleName {
///     const ID_NAME: &str = "exna";
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
    /// Must be overridden with the name of your ID
    const ID_NAME: &'static str;

    /// Provided impl does a compile time check that your ID_NAME is valid. SHOULD NOT BE OVERRIDDEN
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
    // this check is redundant with TNIDName's, but has an important difference:
    // TNIDName's NAME_IS_VALID can be overridden. This could reduce the compile-time strictness.
    // It is, however, nice to still have the check there, as this check in TNID only happens if TNID is used, not necessarily when TNIDName is impled on a type
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
        // put here since this is deemed most unlikely to be refactored
        #![allow(path_statements)] // access causes desired effect: panic on unsatisfied contraint at compile time
        Self::NAME_IS_VALID;

        self.id
    }

    /// Same as [`Self::new_v0`], just a more friendly name
    pub fn new_time_ordered() -> Self {
        Self::new_v0()
    }

    /// Generates a new v0 TNID
    ///
    /// This variant focuses on time sortability, similar to UUIDv7
    pub fn new_v0() -> Self {
        Self::new_v0_with_time(OffsetDateTime::now_utc())
    }

    /// Same as [`Self::new_v1`], just a more friendly name
    pub fn new_high_entropy() -> Self {
        Self::new_v1()
    }

    /// Generates a new v1 TNID
    ///
    /// This variant focuses on maximizing entropy, similar to UUIDv4
    pub fn new_v1() -> Self {
        Self::new_v1_with_random(random())
    }

    /// Generates a new v1 TNID with provided randomness
    pub fn new_v1_with_random(random_bits: u128) -> Self {
        let id_name = Name::ID_NAME;
        debug_assert!(id_name.len() <= name_encoding::NAME_MAX_CHARS);

        let id = v1::make_from_parts(id_name, random_bits);

        Self {
            id_name: PhantomData,
            id,
        }
    }

    pub fn new_v0_with_time(time: OffsetDateTime) -> Self {
        let id_name = Name::ID_NAME;
        debug_assert!(id_name.len() <= name_encoding::NAME_MAX_CHARS);

        let epoch_millis = (time.unix_timestamp_nanos() / 1000 / 1000) as u64;

        let random_bits: u64 = random();

        let id = v0::make_from_parts(id_name, epoch_millis, random_bits);

        Self {
            id_name: PhantomData,
            id,
        }
    }

    pub fn new_v0_with_parts(epoch_millis: u64, random: u64) -> Self {
        Self {
            id_name: PhantomData,
            id: v0::make_from_parts(Name::ID_NAME, epoch_millis, random),
        }
    }

    pub fn as_tnid_string(&self) -> String {
        format!(
            "{}.{}",
            self.name(),
            data_encoding::id_data_to_string(self.id)
        )
    }

    /// Gets the TNID variant
    pub fn variant(&self) -> TNIDVariant {
        let variant_bits = (self.id >> 60) as u8;

        TNIDVariant::from_u8(variant_bits)
    }

    /// Convert to UUID hex string format with specified case
    pub fn to_uuid_string_cased(&self, uppercase: bool) -> String {
        UUIDLike::new(self.id).to_uuid_string_cased(uppercase)
    }

    /// Parse a UUID hex string into a TNID
    pub fn parse_uuid_string(uuid_string: &str) -> Option<Self> {
        let id = UUIDLike::parse_uuid_string(uuid_string)?.as_u128();

        Self::from_u128(id)
    }

    pub fn from_u128(num: u128) -> Option<Self> {
        // check UUIDv8 version and variant bits
        if (num & utils::UUID_V8_MASK) != utils::UUID_V8_MASK {
            return None;
        }

        // check name encoding matches expected name
        let name_bits_mask = 0xFFFFF_u128 << 108; // top 20 bits
        let actual_name_bits = num & name_bits_mask;
        let expected_name_bits = name_encoding::id_name_mask(Name::ID_NAME)?;
        if actual_name_bits != expected_name_bits {
            return None;
        }

        Some(Self {
            id: num,
            id_name: PhantomData,
        })
    }

    #[cfg(feature = "encryption")]
    pub fn encrypt_v0_to_v1(&self, secret: [u8; 16]) -> Result<Self, ()> {
        // Extract only the secret data bits (100 bits, excludes TNID variant)
        let secret_data = encryption::extract_secret_data_bits(self.id);

        // Encrypt the secret data
        let encrypted_data = encryption::encrypt(secret_data, &secret);

        // Expand back to proper bit positions
        let expanded = encryption::expand_secret_data_bits(encrypted_data);

        // Preserve name and UUID metadata, replace data bits with encrypted version
        let id = (self.id & !encryption::COMPLETE_SECRET_DATA_MASK) | expanded;

        // Change variant from V0 to V1
        let id = utils::change_variant(id, TNIDVariant::V1);

        Ok(Self {
            id_name: PhantomData,
            id,
        })
    }

    #[cfg(feature = "encryption")]
    pub fn decrypt_v1_to_v0(&self, secret: [u8; 16]) -> Result<Self, ()> {
        // Extract only the secret data bits (100 bits, excludes TNID variant)
        let encrypted_data = encryption::extract_secret_data_bits(self.id);

        // Decrypt the secret data
        let decrypted_data = encryption::decrypt(encrypted_data, &secret);

        // Expand back to proper bit positions
        let expanded = encryption::expand_secret_data_bits(decrypted_data);

        // Preserve name and UUID metadata, replace data bits with decrypted version
        let id = (self.id & !encryption::COMPLETE_SECRET_DATA_MASK) | expanded;

        // Change variant from V1 to V0
        let id = utils::change_variant(id, TNIDVariant::V0);

        Ok(Self {
            id_name: PhantomData,
            id,
        })
    }
}

impl<Name: TNIDName> std::fmt::Display for TNID<Name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_tnid_string())
    }
}

impl<Name: TNIDName> std::fmt::Debug for TNID<Name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_tnid_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestId;
    impl TNIDName for TestId {
        const ID_NAME: &'static str = "test";
    }

    #[test]
    fn variant0_is_k_sortable() {
        use time::Duration;

        let mut test_time = OffsetDateTime::now_utc();
        let mut last_id: TNID<TestId> = TNID::new_v0_with_time(test_time);

        for _ in 1..10_000 {
            test_time += Duration::milliseconds(1);
            let id: TNID<TestId> = TNID::new_v0_with_time(test_time);

            assert!(last_id.as_u128() < id.as_u128());
            assert!(last_id.as_tnid_string() < id.as_tnid_string());

            last_id = id;
        }
    }

    #[test]
    fn tnid_variant_returns_v0() {
        let id: TNID<TestId> = TNID::new_v0();
        assert_eq!(id.variant(), TNIDVariant::V0);
    }

    #[cfg(feature = "encryption")]
    #[test]
    fn encryption_bidirectional() {}
}
