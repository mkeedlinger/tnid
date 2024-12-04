use rand::random;
use std::marker::PhantomData;
use time::{Date, OffsetDateTime, Time};

mod utils;
mod v0;

pub enum TNIVariant {
    V0,
    V1,
    V2,
    V3,
}

pub trait IdName {
    const ID_NAME: &'static str;
    const NAME_IS_SMALL: () = v0::compile_name_valid_check(Self::ID_NAME);
}

pub struct UUID<Name: IdName> {
    id_name: PhantomData<Name>,
    id: u128,
}

impl<Name: IdName> UUID<Name> {
    pub fn name(&self) -> &'static str {
        Name::ID_NAME
    }

    /// The TNI ID name when the
    /// bytes are encoded using hex.
    pub fn name_hex(&self) -> String {
        let hex = format!("{:05x}", self.id >> 108);

        debug_assert_eq!(hex.len(), 5);

        hex
    }

    pub fn as_u128(&self) -> u128 {
        // put here since this is unlikely to be refactored
        #![allow(path_statements)] // access causes desired effect: panic on unsatisfied contraint at compile time
        Name::NAME_IS_SMALL;

        self.id
    }

    /// Generates
    pub fn new_time_sortable() -> Self {
        Self::new_v0()
    }

    pub fn new_v0() -> Self {
        let id_name = Name::ID_NAME;
        debug_assert!(id_name.len() <= utils::NAME_MAX);

        let now = OffsetDateTime::now_utc();

        let years_since_unix_epoch = {
            let years_since_unix_epoch = now.year() - OffsetDateTime::UNIX_EPOCH.year();

            // going back in time is impossible, right?
            debug_assert!(years_since_unix_epoch > 50);

            // Using a cast instead of .try_into() means that
            // it won't fail if by some craziness it is far in the future
            let years_since_unix_epoch: u8 = years_since_unix_epoch as u8;

            years_since_unix_epoch
        };

        let seconds_since_year_start = {
            let beginning_of_year = OffsetDateTime::new_utc(
                Date::from_calendar_date(now.year(), time::Month::January, 1).unwrap(),
                Time::MIDNIGHT,
            );

            // Using a cast to avoid an unwrap, negatives will be 0
            let seconds_since_year_start: u32 = (now - beginning_of_year).whole_seconds() as u32;

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

    pub fn to_tni_string(&self) -> String {
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

    pub fn tni_variant(&self) -> TNIVariant {
        todo!()
    }
}

impl<KindName: IdName> std::fmt::Display for UUID<KindName> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_tni_string())
    }
}

impl<KindName: IdName> std::fmt::Debug for UUID<KindName> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_tni_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Time Error: {0:?}")]
    TimeError(Box<dyn std::error::Error>),
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestId;
    impl IdName for TestId {
        const ID_NAME: &'static str = "test";
    }

    #[test]
    fn variant0_is_k_sortable() {
        let mut last_id: UUID<TestId> = UUID::new_v0();

        for _ in 0..5 {
            std::thread::sleep(std::time::Duration::from_millis(1_010));

            let id: UUID<TestId> = UUID::new_v0();

            dbg!(&last_id, &id);
            assert!(last_id.as_u128() < id.as_u128());
            assert!(last_id.to_tni_string() < id.to_tni_string());

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
}
