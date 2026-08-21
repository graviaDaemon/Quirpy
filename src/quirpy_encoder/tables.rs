use std::sync::LazyLock;

use super::{EcLevel, Mode};
use crate::quirpy_encoder::matrix;

pub const ALPHANUMERIC: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

pub const TERMINATOR_BITS: usize = 4;
pub const PAD_BYTES: [u8; 2] = [0xEC, 0x11];

// Table E.1 — alignment pattern centre coordinates.
pub const ALIGNMENT_CENTRES: [&[u8]; 40] = [
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];

// Table 9 — (error correction codewords per block, block count) per version, in L, M, Q, H order.
pub const EC_BLOCKS: [[(u8, u8); 4]; 40] = [
    [(7, 1), (10, 1), (13, 1), (17, 1)],
    [(10, 1), (16, 1), (22, 1), (28, 1)],
    [(15, 1), (26, 1), (18, 2), (22, 2)],
    [(20, 1), (18, 2), (26, 2), (16, 4)],
    [(26, 1), (24, 2), (18, 4), (22, 4)],
    [(18, 2), (16, 4), (24, 4), (28, 4)],
    [(20, 2), (18, 4), (18, 6), (26, 5)],
    [(24, 2), (22, 4), (22, 6), (26, 6)],
    [(30, 2), (22, 5), (20, 8), (24, 8)],
    [(18, 4), (26, 5), (24, 8), (28, 8)],
    [(20, 4), (30, 5), (28, 8), (24, 11)],
    [(24, 4), (22, 8), (26, 10), (28, 11)],
    [(26, 4), (22, 9), (24, 12), (22, 16)],
    [(30, 4), (24, 9), (20, 16), (24, 16)],
    [(22, 6), (24, 10), (30, 12), (24, 18)],
    [(24, 6), (28, 10), (24, 17), (30, 16)],
    [(28, 6), (28, 11), (28, 16), (28, 19)],
    [(30, 6), (26, 13), (28, 18), (28, 21)],
    [(28, 7), (26, 14), (26, 21), (26, 25)],
    [(28, 8), (26, 16), (30, 20), (28, 25)],
    [(28, 8), (26, 17), (28, 23), (30, 25)],
    [(28, 9), (28, 17), (30, 23), (24, 34)],
    [(30, 9), (28, 18), (30, 25), (30, 30)],
    [(30, 10), (28, 20), (30, 27), (30, 32)],
    [(26, 12), (28, 21), (30, 29), (30, 35)],
    [(28, 12), (28, 23), (28, 34), (30, 37)],
    [(30, 12), (28, 25), (30, 34), (30, 40)],
    [(30, 13), (28, 26), (30, 35), (30, 42)],
    [(30, 14), (28, 28), (30, 38), (30, 45)],
    [(30, 15), (28, 29), (30, 40), (30, 48)],
    [(30, 16), (28, 31), (30, 43), (30, 51)],
    [(30, 17), (28, 33), (30, 45), (30, 54)],
    [(30, 18), (28, 35), (30, 48), (30, 57)],
    [(30, 19), (28, 37), (30, 51), (30, 60)],
    [(30, 19), (28, 38), (30, 53), (30, 63)],
    [(30, 20), (28, 40), (30, 56), (30, 66)],
    [(30, 21), (28, 43), (30, 59), (30, 70)],
    [(30, 22), (28, 45), (30, 62), (30, 74)],
    [(30, 24), (28, 47), (30, 65), (30, 77)],
    [(30, 25), (28, 49), (30, 68), (30, 81)],
];

static CAPACITY: LazyLock<[(u16, u8); 40]> = LazyLock::new(|| {
    let mut capacity = [(0u16, 0u8); 40];
    for (index, slot) in capacity.iter_mut().enumerate() {
        let free = matrix::free_modules(index as u8 + 1);
        *slot = ((free / 8) as u16, (free % 8) as u8);
    }
    capacity
});

pub fn total_codewords(version: u8) -> usize {
    CAPACITY[version as usize - 1].0 as usize
}

pub fn remainder_bits(version: u8) -> usize {
    CAPACITY[version as usize - 1].1 as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub ec_per_block: usize,
    pub blocks: usize,
    pub short_len: usize,
    pub short_count: usize,
    pub long_count: usize,
}

impl Split {
    pub fn data_total(&self) -> usize {
        self.short_count * self.short_len + self.long_count * (self.short_len + 1)
    }
}

pub fn split(version: u8, ec: EcLevel) -> Split {
    let (ec_per_block, blocks) = EC_BLOCKS[version as usize - 1][ec.index()];
    let (ec_per_block, blocks) = (ec_per_block as usize, blocks as usize);

    let data_total = total_codewords(version) - ec_per_block * blocks;
    let short_len = data_total / blocks;
    let long_count = data_total % blocks;

    Split {
        ec_per_block,
        blocks,
        short_len,
        short_count: blocks - long_count,
        long_count,
    }
}

pub fn capacity_bits(version: u8, ec: EcLevel) -> usize {
    split(version, ec).data_total() * 8
}

pub fn character_count_bits(version: u8, mode: Mode) -> usize {
    match (version, mode) {
        (1..=9, Mode::Numeric) => 10,
        (1..=9, Mode::Alphanumeric) => 9,
        (1..=9, Mode::Byte) => 8,
        (10..=26, Mode::Numeric) => 12,
        (10..=26, Mode::Alphanumeric) => 11,
        (_, Mode::Numeric) => 14,
        (_, Mode::Alphanumeric) => 13,
        (_, Mode::Byte) => 16,
    }
}

pub fn max_characters(version: u8, ec: EcLevel, mode: Mode) -> usize {
    let header = 4 + character_count_bits(version, mode);
    let available = capacity_bits(version, ec).saturating_sub(header);

    match mode {
        Mode::Numeric => {
            let mut count = available / 10 * 3;
            match available % 10 {
                7..=9 => count += 2,
                4..=6 => count += 1,
                _ => {}
            }
            count
        }
        Mode::Alphanumeric => {
            let mut count = available / 11 * 2;
            if available % 11 >= 6 {
                count += 1;
            }
            count
        }
        Mode::Byte => available / 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_totals_match_the_published_figures() {
        assert_eq!((total_codewords(1), remainder_bits(1)), (26, 0));
        assert_eq!((total_codewords(2), remainder_bits(2)), (44, 7));
        assert_eq!((total_codewords(7), remainder_bits(7)), (196, 0));
        assert_eq!((total_codewords(40), remainder_bits(40)), (3706, 0));
    }

    #[test]
    fn alignment_centres_hold_their_invariants() {
        for version in 2..=40u8 {
            let centres = ALIGNMENT_CENTRES[version as usize - 1];
            assert_eq!(centres[0], 6, "version {version}");
            assert_eq!(
                *centres.last().expect("non-empty"),
                4 * version + 10,
                "version {version}"
            );
            assert_eq!(centres.len(), version as usize / 7 + 2, "version {version}");
            for centre in &centres[1..] {
                assert_eq!(centre % 2, 0, "version {version} centre {centre}");
            }
        }
        assert!(ALIGNMENT_CENTRES[0].is_empty());
    }

    #[test]
    fn every_block_split_is_possible() {
        for version in 1..=40u8 {
            for ec in EcLevel::ALL {
                let split = split(version, ec);
                assert!(
                    split.ec_per_block * split.blocks < total_codewords(version),
                    "version {version} {ec:?}"
                );
                assert!(
                    split.data_total() >= split.blocks,
                    "version {version} {ec:?}"
                );
                assert_eq!(
                    split.data_total() + split.ec_per_block * split.blocks,
                    total_codewords(version),
                    "version {version} {ec:?}"
                );
            }
        }
    }

    #[test]
    fn version_one_data_codewords_match_the_spec() {
        let expected = [19, 16, 13, 9];
        for (ec, want) in EcLevel::ALL.into_iter().zip(expected) {
            let split = split(1, ec);
            assert_eq!(split.blocks, 1, "{ec:?}");
            assert_eq!(split.data_total(), want, "{ec:?}");
        }
    }

    #[test]
    fn byte_capacity_matches_the_published_table_at_the_extremes() {
        let first = [17, 14, 11, 7];
        let last = [2953, 2331, 1663, 1273];
        for (index, ec) in EcLevel::ALL.into_iter().enumerate() {
            assert_eq!(max_characters(1, ec, Mode::Byte), first[index], "{ec:?}");
            assert_eq!(max_characters(40, ec, Mode::Byte), last[index], "{ec:?}");
        }
    }

    #[test]
    fn text_capacity_matches_the_published_table_at_version_one() {
        assert_eq!(max_characters(1, EcLevel::L, Mode::Alphanumeric), 25);
        assert_eq!(max_characters(1, EcLevel::L, Mode::Numeric), 41);
    }

    #[test]
    fn the_alphanumeric_table_is_the_spec_ordering() {
        assert_eq!(ALPHANUMERIC.len(), 45);
        assert_eq!(ALPHANUMERIC.find('0'), Some(0));
        assert_eq!(ALPHANUMERIC.find('A'), Some(10));
        assert_eq!(ALPHANUMERIC.find(' '), Some(36));
        assert_eq!(ALPHANUMERIC.find(':'), Some(44));
    }
}
