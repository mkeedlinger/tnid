pub const fn name_valid_check(name: &str) {
    if let NAME_MIN_CHARS..=NAME_MAX_CHARS = name.len() {
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
        while j < CHAR_MAPPING.len() {
            if CHAR_MAPPING[j].1 == bytes[i] {
                i += 1;
                continue 'check_loop;
            }
            j += 1;
        }

        panic!("Invalid char in name");
    }
}

pub const NAME_MIN_CHARS: usize = 1;
pub const NAME_MAX_CHARS: usize = 4;

pub const CHAR_BIT_LENGTH: u8 = 5;

pub const CHAR_MAPPING: [(u8, u8); 31] = [
    // zero is a null terminator

    // nums
    (1, b'0'),
    (2, b'1'),
    (3, b'2'),
    (4, b'3'),
    (5, b'4'),
    // alpha
    (6, b'a'),
    (7, b'b'),
    (8, b'c'),
    (9, b'd'),
    (10, b'e'),
    (11, b'f'),
    (12, b'g'),
    (13, b'h'),
    (14, b'i'),
    (15, b'j'),
    (16, b'k'),
    (17, b'l'),
    (18, b'm'),
    (19, b'n'),
    (20, b'o'),
    (21, b'p'),
    (22, b'q'),
    (23, b'r'),
    (24, b's'),
    (25, b't'),
    (26, b'u'),
    (27, b'v'),
    (28, b'w'),
    (29, b'x'),
    (30, b'y'),
    (31, b'z'),
];

pub fn id_name_mask(name: &str) -> Option<u128> {
    if !name.is_ascii() {
        return None;
    }

    let name_bytes = name.as_bytes();

    if name_bytes.len() > NAME_MAX_CHARS {
        return None;
    }

    let mut mask = 0u128;

    for &name_char in name.as_bytes() {
        let encoding_mapping = CHAR_MAPPING
            .iter()
            .find(|(_encoded, from_char)| *from_char == name_char);

        let Some((encoded_byte, _)) = encoding_mapping else {
            // mapping not possible
            return None;
        };

        debug_assert!(*encoded_byte < 32);

        mask <<= CHAR_BIT_LENGTH;
        mask |= *encoded_byte as u128;
    }

    let needed_padding_chars = NAME_MAX_CHARS - name.len();
    mask <<= CHAR_BIT_LENGTH * needed_padding_chars as u8;

    mask <<= 108;

    Some(mask)
}
