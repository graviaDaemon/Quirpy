mod blocks;
mod format;
mod galois;
mod mask;
mod matrix;
mod segment;
mod tables;

use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EcLevel {
    L,
    #[default]
    M,
    Q,
    H,
}

impl EcLevel {
    pub const ALL: [Self; 4] = [Self::L, Self::M, Self::Q, Self::H];

    pub fn label(self) -> &'static str {
        match self {
            Self::L => "Low (L)",
            Self::M => "Medium (M)",
            Self::Q => "Quartile (Q)",
            Self::H => "High (H)",
        }
    }

    pub fn keyword(self) -> &'static str {
        match self {
            Self::L => "l",
            Self::M => "m",
            Self::Q => "q",
            Self::H => "h",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "l" | "low" => Some(Self::L),
            "m" | "medium" => Some(Self::M),
            "q" | "quartile" => Some(Self::Q),
            "h" | "high" => Some(Self::H),
            _ => None,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::L => 0,
            Self::M => 1,
            Self::Q => 2,
            Self::H => 3,
        }
    }

    fn format_bits(self) -> u32 {
        match self {
            Self::L => 0b01,
            Self::M => 0b00,
            Self::Q => 0b11,
            Self::H => 0b10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Mode::Numeric => "Numeric",
            Mode::Alphanumeric => "Alphanumeric",
            Mode::Byte => "Byte",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub matrix: Vec<Vec<bool>>,
    pub version: u8,
    pub ec: EcLevel,
    pub mode: Mode,
    pub mask: u8,
}

impl Symbol {
    pub fn size(&self) -> usize {
        self.matrix.len()
    }

    pub fn dark_modules(&self) -> usize {
        self.matrix
            .iter()
            .flatten()
            .filter(|module| **module)
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    TooLong {
        mode: Mode,
        length: usize,
        max: usize,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::TooLong { mode, length, max } => {
                let unit = match mode {
                    Mode::Byte => "bytes",
                    _ => "characters",
                };
                write!(
                    f,
                    "Payload is too long: {length} {unit}, and the largest QR code holds {max} at \
                     this error correction level."
                )
            }
        }
    }
}

impl std::error::Error for EncodeError {}

pub fn encode(payload: &str, ec: EcLevel) -> Result<Symbol, EncodeError> {
    let mode = Mode::detect(payload);
    let version = segment::choose_version(mode, payload, ec)?;

    let data = segment::encode(payload, mode, version, ec);
    let stream = blocks::interleave(&data, version, ec);

    let mut base = matrix::build(version);
    matrix::place_data(&mut base, &stream);

    let candidates: Vec<matrix::Modules> = (0..mask::COUNT)
        .map(|mask| finish(&base, version, ec, mask))
        .collect();
    let mask = candidates
        .iter()
        .enumerate()
        .min_by_key(|(_, modules)| mask::penalty(&modules.dark))
        .map(|(mask, _)| mask as u8)
        .expect("eight mask candidates");

    tracing::debug!(
        version,
        ?mode,
        ?ec,
        mask,
        bytes = payload.len(),
        "encoded symbol"
    );

    Ok(Symbol {
        matrix: candidates[mask as usize].dark.clone(),
        version,
        ec,
        mode,
        mask,
    })
}

fn finish(base: &matrix::Modules, version: u8, ec: EcLevel, mask: u8) -> matrix::Modules {
    let mut modules = base.clone();
    mask::apply(&mut modules, mask);
    format::place_format(&mut modules, ec, mask);
    if version >= 7 {
        format::place_version(&mut modules, version);
    }
    modules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world_is_a_version_one_alphanumeric_symbol() {
        let symbol = encode("HELLO WORLD", EcLevel::Q).expect("fits version 1");

        assert_eq!(symbol.version, 1);
        assert_eq!(symbol.mode, Mode::Alphanumeric);
        assert_eq!(symbol.size(), 21);
        assert!(symbol.matrix.iter().all(|row| row.len() == 21));
        assert!(symbol.mask < 8);
    }

    #[test]
    fn the_spec_worked_example_is_a_version_one_numeric_symbol() {
        let symbol = encode("01234567", EcLevel::M).expect("fits version 1");

        assert_eq!(symbol.version, 1);
        assert_eq!(symbol.mode, Mode::Numeric);
        assert_eq!(symbol.size(), 21);
    }

    #[test]
    fn an_empty_payload_still_produces_a_symbol() {
        let symbol = encode("", EcLevel::M).expect("empty fits version 1");
        assert_eq!(symbol.version, 1);
        assert_eq!(symbol.size(), 21);
    }

    #[test]
    fn every_finished_symbol_carries_its_function_patterns() {
        for version in [1u8, 6, 7, 9, 10, 26, 27, 40] {
            let size = matrix::size(version);
            let payload = "A".repeat(tables::max_characters(
                version,
                EcLevel::L,
                Mode::Alphanumeric,
            ));
            let symbol = encode(&payload, EcLevel::L).expect("generated payload fits");

            assert_eq!(symbol.version, version, "version {version}");
            assert_eq!(symbol.size(), size, "version {version}");

            for (row, col) in [(0, 0), (0, size - 7), (size - 7, 0)] {
                assert!(symbol.matrix[row][col], "finder corner, version {version}");
                assert!(
                    !symbol.matrix[row + 1][col + 1],
                    "finder ring, version {version}"
                );
                assert!(
                    symbol.matrix[row + 3][col + 3],
                    "finder core, version {version}"
                );
            }

            assert!(symbol.matrix[size - 8][8], "dark module, version {version}");
            for col in 8..size - 8 {
                assert_eq!(
                    symbol.matrix[6][col],
                    col % 2 == 0,
                    "timing, version {version}"
                );
            }
        }
    }

    #[test]
    fn the_version_boundaries_encode_at_every_level() {
        for version in [1u8, 9, 10, 26, 27, 40] {
            for ec in EcLevel::ALL {
                for mode in [Mode::Numeric, Mode::Alphanumeric, Mode::Byte] {
                    let max = tables::max_characters(version, ec, mode);
                    let payload: String = match mode {
                        Mode::Numeric => "7".repeat(max),
                        Mode::Alphanumeric => "A".repeat(max),
                        Mode::Byte => "a".repeat(max),
                    };
                    let symbol = encode(&payload, ec).expect("maximum-length payload fits");
                    assert_eq!(
                        symbol.version, version,
                        "{ec:?} {mode} at version {version}"
                    );
                    assert_eq!(symbol.mode, mode, "{ec:?} {mode} at version {version}");
                }
            }
        }
    }

    #[test]
    fn a_payload_past_the_largest_symbol_is_rejected() {
        let payload = "a".repeat(3000);
        assert!(matches!(
            encode(&payload, EcLevel::H),
            Err(EncodeError::TooLong { .. })
        ));
    }

    #[test]
    fn a_higher_error_correction_level_never_shrinks_the_symbol() {
        let payload = "https://example.com/some/reasonably/long/path?query=value";
        let mut previous = 0;
        for ec in EcLevel::ALL {
            let symbol = encode(payload, ec).expect("fits");
            assert!(symbol.version >= previous, "{ec:?}");
            previous = symbol.version;
        }
    }

    // The encoder is validated against published codeword vectors rather than a reference
    // implementation, so this reads the data stream back out of the finished symbol to prove
    // masking and placement are reversible in the order a scanner walks them.
    #[test]
    fn the_finished_symbol_round_trips_back_to_its_codewords() {
        for (payload, ec) in [
            ("HELLO WORLD", EcLevel::Q),
            ("01234567", EcLevel::M),
            ("https://example.com", EcLevel::H),
            (
                "BEGIN:VCARD VERSION:3.0 N:DE VRIES;JAN TEL:+31612345678 END:VCARD",
                EcLevel::M,
            ),
        ] {
            let symbol = encode(payload, ec).expect("fits");

            let mut recovered = matrix::build(symbol.version);
            recovered.dark = symbol.matrix.clone();
            if symbol.version >= 7 {
                format::place_version(&mut recovered, symbol.version);
            }
            mask::apply(&mut recovered, symbol.mask);

            let mut bits = Vec::new();
            let size = recovered.size;
            let mut upward = true;
            let mut right = size - 1;
            loop {
                if right == 6 {
                    right -= 1;
                }
                for step in 0..size {
                    let row = if upward { size - 1 - step } else { step };
                    for col in [right, right - 1] {
                        if !recovered.reserved[row][col] {
                            bits.push(recovered.dark[row][col]);
                        }
                    }
                }
                upward = !upward;
                if right < 3 {
                    break;
                }
                right -= 2;
            }

            let data = segment::encode(payload, symbol.mode, symbol.version, ec);
            let expected = blocks::interleave(&data, symbol.version, ec);
            assert_eq!(bits, expected, "{payload:?} at {ec:?}");
        }
    }
}
