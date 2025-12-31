/// The 4 possible TNID variants
///
/// Similar to UUID variants, TNID variants have different construction that makes them useful for different situations.
#[derive(Debug, PartialEq, Eq)]
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

impl TNIDVariant {
    /// Convert a u8 to a [`TNIDVariant`].
    ///
    /// Ignores the top 6 bits
    pub fn from_u8(variant_bits: u8) -> TNIDVariant {
        let variant_bits = variant_bits & 0b11;

        match variant_bits {
            0 => TNIDVariant::V0,
            1 => TNIDVariant::V1,
            2 => TNIDVariant::V2,
            3 => TNIDVariant::V3,
            _ => unreachable!("2 bits can only have 4 values"),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            TNIDVariant::V0 => 0,
            TNIDVariant::V1 => 1,
            TNIDVariant::V2 => 2,
            TNIDVariant::V3 => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u8_no_panic() {
        for i in u8::MIN..=u8::MAX {
            TNIDVariant::from_u8(i);
        }
    }
}
