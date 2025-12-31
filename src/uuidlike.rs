/// Used when you have 128 bits that *could* be a TNID, but might not be and you want to inspect why it's not conformant
pub struct UUIDLike(u128);

impl UUIDLike {
    pub fn as_u128(&self) -> u128 {
        self.0
    }

    pub fn new(id: u128) -> Self {
        Self(id)
    }

    /// Convert to UUID hex string format with specified case
    pub fn to_uuid_string_cased(&self, uppercase: bool) -> String {
        let id = self.0;

        // Format as UUID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let first_section = (id >> 96) as u32;
        let second_section = ((id >> 80) & 0xffff) as u16;
        let third_section = ((id >> 64) & 0xffff) as u16;
        let fourth_section = ((id >> 48) & 0xffff) as u16;
        let fifth_section = (id & 0xffffffffffff) as u64;

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

    pub fn parse_uuid_string(uuid_string: &str) -> Option<Self> {
        if uuid_string.len() != 36 {
            return None;
        }

        let bytes = uuid_string.as_bytes();
        if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
            return None;
        }

        // the from_str_radix below should also check that chars are hex digits, so this is redundant, but included for easier debugging
        #[cfg(debug_assertions)]
        for (i, c) in uuid_string.chars().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                if c != '-' {
                    return None;
                }
            } else if !c.is_ascii_hexdigit() {
                return None;
            }
        }

        // parse 5 hyphen-separated sections as hex
        let s1 = u32::from_str_radix(&uuid_string[0..8], 16).ok()?;
        let s2 = u16::from_str_radix(&uuid_string[9..13], 16).ok()?;
        let s3 = u16::from_str_radix(&uuid_string[14..18], 16).ok()?;
        let s4 = u16::from_str_radix(&uuid_string[19..23], 16).ok()?;
        let s5 = u64::from_str_radix(&uuid_string[24..36], 16).ok()?;

        // Combine sections into u128 (reverse of to_uuid_string_cased)
        let id = ((s1 as u128) << 96)
            | ((s2 as u128) << 80)
            | ((s3 as u128) << 64)
            | ((s4 as u128) << 48)
            | (s5 as u128);

        Some(Self(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lowercase() {
        let result = UUIDLike::parse_uuid_string("ffffffff-ffff-ffff-ffff-ffffffffffff");
        assert_eq!(result.unwrap().as_u128(), u128::MAX);
    }

    #[test]
    fn parse_uppercase() {
        let result = UUIDLike::parse_uuid_string("FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF");
        assert_eq!(result.unwrap().as_u128(), u128::MAX);
    }

    #[test]
    fn parse_mixed_case() {
        let result = UUIDLike::parse_uuid_string("AaBbCcDd-1234-5678-90aB-cDeF01234567");
        assert!(result.is_some());
    }

    #[test]
    fn parse_all_zeros() {
        let result = UUIDLike::parse_uuid_string("00000000-0000-0000-0000-000000000000");
        assert_eq!(result.unwrap().as_u128(), 0);
    }
}
