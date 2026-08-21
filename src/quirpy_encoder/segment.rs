use super::tables::{self, ALPHANUMERIC, PAD_BYTES, TERMINATOR_BITS};
use super::{EcLevel, EncodeError, Mode};

impl Mode {
    pub fn detect(payload: &str) -> Self {
        if !payload.is_empty() && payload.bytes().all(|byte| byte.is_ascii_digit()) {
            Mode::Numeric
        } else if !payload.is_empty() && payload.chars().all(|c| ALPHANUMERIC.contains(c)) {
            Mode::Alphanumeric
        } else {
            Mode::Byte
        }
    }

    pub fn indicator(self) -> u32 {
        match self {
            Mode::Numeric => 0b0001,
            Mode::Alphanumeric => 0b0010,
            Mode::Byte => 0b0100,
        }
    }

    fn count(self, payload: &str) -> usize {
        match self {
            Mode::Numeric | Mode::Alphanumeric => payload.chars().count(),
            Mode::Byte => payload.len(),
        }
    }
}

#[derive(Default)]
pub struct BitBuffer {
    bits: Vec<bool>,
}

impl BitBuffer {
    pub fn push_bits(&mut self, value: u32, len: usize) {
        for shift in (0..len).rev() {
            self.bits.push(value >> shift & 1 == 1);
        }
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    fn into_codewords(self) -> Vec<u8> {
        self.bits
            .chunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0u8, |byte, bit| byte << 1 | u8::from(*bit))
            })
            .collect()
    }
}

pub fn choose_version(mode: Mode, payload: &str, ec: EcLevel) -> Result<u8, EncodeError> {
    let count = mode.count(payload);
    for version in 1..=40u8 {
        if count <= tables::max_characters(version, ec, mode) {
            return Ok(version);
        }
    }
    Err(EncodeError::TooLong {
        mode,
        length: count,
        max: tables::max_characters(40, ec, mode),
    })
}

pub fn encode(payload: &str, mode: Mode, version: u8, ec: EcLevel) -> Vec<u8> {
    let capacity = tables::capacity_bits(version, ec);
    let mut buffer = BitBuffer::default();

    buffer.push_bits(mode.indicator(), 4);
    buffer.push_bits(
        mode.count(payload) as u32,
        tables::character_count_bits(version, mode),
    );

    match mode {
        Mode::Numeric => numeric(&mut buffer, payload),
        Mode::Alphanumeric => alphanumeric(&mut buffer, payload),
        Mode::Byte => {
            for byte in payload.bytes() {
                buffer.push_bits(byte as u32, 8);
            }
        }
    }

    let terminator = TERMINATOR_BITS.min(capacity.saturating_sub(buffer.len()));
    buffer.push_bits(0, terminator);
    buffer.push_bits(0, buffer.len().next_multiple_of(8) - buffer.len());

    let mut codewords = buffer.into_codewords();
    for pad in PAD_BYTES.into_iter().cycle() {
        if codewords.len() >= capacity / 8 {
            break;
        }
        codewords.push(pad);
    }

    codewords
}

fn numeric(buffer: &mut BitBuffer, payload: &str) {
    let digits: Vec<u32> = payload.bytes().map(|byte| u32::from(byte - b'0')).collect();

    for group in digits.chunks(3) {
        let value = group.iter().fold(0, |value, digit| value * 10 + digit);
        buffer.push_bits(
            value,
            match group.len() {
                3 => 10,
                2 => 7,
                _ => 4,
            },
        );
    }
}

fn alphanumeric(buffer: &mut BitBuffer, payload: &str) {
    let values: Vec<u32> = payload
        .chars()
        .map(|c| ALPHANUMERIC.find(c).expect("checked by Mode::detect") as u32)
        .collect();

    for pair in values.chunks(2) {
        match pair {
            [first, second] => buffer.push_bits(first * 45 + second, 11),
            [only] => buffer.push_bits(*only, 6),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_detection_picks_the_narrowest_mode() {
        assert_eq!(Mode::detect("12345"), Mode::Numeric);
        assert_eq!(Mode::detect("HELLO WORLD"), Mode::Alphanumeric);
        assert_eq!(Mode::detect("hello"), Mode::Byte);
        assert_eq!(Mode::detect("https://example.com"), Mode::Byte);
        assert_eq!(Mode::detect("HTTPS://EXAMPLE.COM"), Mode::Alphanumeric);
        assert_eq!(Mode::detect(""), Mode::Byte);
    }

    // ISO/IEC 18004 Annex I worked example: version 1-M, payload "01234567".
    #[test]
    fn the_spec_worked_example_produces_its_published_data_codewords() {
        assert_eq!(
            encode("01234567", Mode::Numeric, 1, EcLevel::M),
            vec![
                0x10, 0x20, 0x0C, 0x56, 0x61, 0x80, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11,
                0xEC, 0x11
            ]
        );
    }

    // Published worked example: "HELLO WORLD" at version 1-Q.
    #[test]
    fn the_hello_world_example_produces_its_published_data_codewords() {
        assert_eq!(
            encode("HELLO WORLD", Mode::Alphanumeric, 1, EcLevel::Q),
            vec![32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236]
        );
    }

    #[test]
    fn the_data_block_is_always_filled_exactly() {
        for version in [1u8, 9, 10, 26, 27, 40] {
            for ec in EcLevel::ALL {
                for mode in [Mode::Numeric, Mode::Alphanumeric, Mode::Byte] {
                    let max = tables::max_characters(version, ec, mode);
                    let payload: String = match mode {
                        Mode::Numeric => "7".repeat(max),
                        Mode::Alphanumeric => "A".repeat(max),
                        Mode::Byte => "a".repeat(max),
                    };
                    assert_eq!(
                        encode(&payload, mode, version, ec).len(),
                        tables::capacity_bits(version, ec) / 8,
                        "version {version} {ec:?} {mode}"
                    );
                }
            }
        }
    }

    #[test]
    fn version_selection_climbs_only_as_far_as_it_must() {
        assert_eq!(choose_version(Mode::Numeric, "01234567", EcLevel::M), Ok(1));
        assert_eq!(
            choose_version(Mode::Alphanumeric, "HELLO WORLD", EcLevel::Q),
            Ok(1)
        );
        assert_eq!(
            choose_version(Mode::Byte, &"a".repeat(18), EcLevel::L),
            Ok(2)
        );
    }

    #[test]
    fn one_character_past_version_forty_is_rejected() {
        let payload = "a".repeat(tables::max_characters(40, EcLevel::L, Mode::Byte) + 1);
        assert_eq!(
            choose_version(Mode::Byte, &payload, EcLevel::L),
            Err(EncodeError::TooLong {
                mode: Mode::Byte,
                length: 2954,
                max: 2953,
            })
        );
    }

    #[test]
    fn an_empty_payload_fits_version_one() {
        assert_eq!(choose_version(Mode::Byte, "", EcLevel::M), Ok(1));
        assert_eq!(
            encode("", Mode::Byte, 1, EcLevel::M).len(),
            tables::capacity_bits(1, EcLevel::M) / 8
        );
    }
}
