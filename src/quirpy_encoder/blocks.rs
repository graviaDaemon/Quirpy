use super::EcLevel;
use super::galois;
use super::tables;

pub fn interleave(data: &[u8], version: u8, ec: EcLevel) -> Vec<bool> {
    let split = tables::split(version, ec);
    let generator = galois::generator_poly(split.ec_per_block);

    let mut data_blocks: Vec<&[u8]> = Vec::with_capacity(split.blocks);
    let mut offset = 0;
    for index in 0..split.blocks {
        let len = if index < split.short_count {
            split.short_len
        } else {
            split.short_len + 1
        };
        data_blocks.push(&data[offset..offset + len]);
        offset += len;
    }

    let ec_blocks: Vec<Vec<u8>> = data_blocks
        .iter()
        .map(|block| galois::remainder(block, &generator))
        .collect();

    let mut stream = Vec::with_capacity(tables::total_codewords(version));
    for index in 0..split.short_len + 1 {
        for block in &data_blocks {
            if let Some(codeword) = block.get(index) {
                stream.push(*codeword);
            }
        }
    }
    for index in 0..split.ec_per_block {
        for block in &ec_blocks {
            stream.push(block[index]);
        }
    }

    let mut bits = Vec::with_capacity(stream.len() * 8 + tables::remainder_bits(version));
    for codeword in stream {
        for shift in (0..8).rev() {
            bits.push(codeword >> shift & 1 == 1);
        }
    }
    bits.resize(bits.len() + tables::remainder_bits(version), false);
    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quirpy_encoder::{Mode, segment};

    fn codewords(bits: &[bool]) -> Vec<u8> {
        bits.chunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0u8, |byte, bit| byte << 1 | u8::from(*bit))
            })
            .collect()
    }

    // ISO/IEC 18004 Annex I worked example: version 1-M, payload "01234567".
    #[test]
    fn the_spec_worked_example_produces_its_published_ec_codewords() {
        let data = segment::encode("01234567", Mode::Numeric, 1, EcLevel::M);
        let stream = codewords(&interleave(&data, 1, EcLevel::M));

        assert_eq!(&stream[..16], &data[..]);
        assert_eq!(
            &stream[16..],
            &[0xA5, 0x24, 0xD4, 0xC1, 0xED, 0x36, 0xC7, 0x87, 0x2C, 0x55]
        );
    }

    // Every version-1 symbol is a single block, so the data half must come through untouched and
    // the error correction codewords must simply follow it. The degree-13 generator this exercises
    // is checked against ISO/IEC 18004 Table A.1 in galois.rs.
    #[test]
    fn a_single_block_version_interleaves_to_the_identity() {
        for ec in EcLevel::ALL {
            let data = segment::encode("HELLO", Mode::Alphanumeric, 1, ec);
            let stream = codewords(&interleave(&data, 1, ec));
            assert_eq!(&stream[..data.len()], &data[..], "{ec:?}");
            assert_eq!(stream.len(), tables::total_codewords(1), "{ec:?}");
        }
    }

    #[test]
    fn every_version_fills_the_symbol_exactly() {
        for version in 1..=40u8 {
            for ec in EcLevel::ALL {
                let split = tables::split(version, ec);
                let data = vec![0x42u8; split.data_total()];
                let bits = interleave(&data, version, ec);
                assert_eq!(
                    bits.len(),
                    tables::total_codewords(version) * 8 + tables::remainder_bits(version),
                    "version {version} {ec:?}"
                );
            }
        }
    }

    #[test]
    fn interleaving_is_a_permutation_of_the_data_codewords() {
        let version = 13u8;
        let ec = EcLevel::H;
        let split = tables::split(version, ec);
        let data: Vec<u8> = (0..split.data_total()).map(|index| index as u8).collect();

        let stream = codewords(&interleave(&data, version, ec));
        let mut carried: Vec<u8> = stream[..data.len()].to_vec();
        carried.sort_unstable();
        let mut expected = data.clone();
        expected.sort_unstable();
        assert_eq!(carried, expected);
    }
}
