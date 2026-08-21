use super::EcLevel;
use super::matrix::Modules;

const FORMAT_GENERATOR: u32 = 0b101_0011_0111;
const FORMAT_XOR: u32 = 0b101_0100_0001_0010;
const VERSION_GENERATOR: u32 = 0b1_1111_0010_0101;

pub fn format_bits(ec: EcLevel, mask: u8) -> u32 {
    let data = ec.format_bits() << 3 | u32::from(mask);
    (data << 10 | bch(data << 10, FORMAT_GENERATOR)) ^ FORMAT_XOR
}

pub fn version_bits(version: u8) -> u32 {
    let data = u32::from(version);
    data << 12 | bch(data << 12, VERSION_GENERATOR)
}

fn bch(mut value: u32, generator: u32) -> u32 {
    let width = u32::BITS - generator.leading_zeros();
    while u32::BITS - value.leading_zeros() >= width {
        value ^= generator << (u32::BITS - value.leading_zeros() - width);
    }
    value
}

const TOP_LEFT: [(usize, usize); 15] = [
    (0, 8),
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (7, 8),
    (8, 8),
    (8, 7),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
    (8, 0),
];

pub fn place_format(modules: &mut Modules, ec: EcLevel, mask: u8) {
    let bits = format_bits(ec, mask);
    let last = modules.size - 1;
    let on = |index: u32| bits >> index & 1 == 1;

    // The two copies run in opposite directions: the top-left one starts at its least significant
    // bit, the split one at its most significant.
    for (index, (row, col)) in TOP_LEFT.into_iter().enumerate() {
        modules.dark[row][col] = on(index as u32);
    }
    for index in 0..7u32 {
        modules.dark[last - index as usize][8] = on(14 - index);
    }
    for index in 7..15u32 {
        modules.dark[8][last - 14 + index as usize] = on(14 - index);
    }
}

pub fn place_version(modules: &mut Modules, version: u8) {
    let bits = version_bits(version);
    let last = modules.size - 1;

    for index in 0..18usize {
        let on = bits >> index & 1 == 1;
        let (row, col) = (index / 3, index % 3);
        modules.dark[last - 10 + col][row] = on;
        modules.dark[row][last - 10 + col] = on;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medium_with_mask_zero_is_the_bare_xor_mask() {
        assert_eq!(format_bits(EcLevel::M, 0), 0b101_0100_0001_0010);
    }

    #[test]
    fn low_with_mask_zero_matches_the_spec() {
        assert_eq!(format_bits(EcLevel::L, 0), 0b111_0111_1100_0100);
    }

    #[test]
    fn version_seven_matches_the_spec() {
        assert_eq!(version_bits(7), 0b00_0111_1100_1001_0100);
    }

    fn distance(a: u32, b: u32) -> u32 {
        (a ^ b).count_ones()
    }

    #[test]
    fn every_format_string_is_seven_bits_from_every_other() {
        let all: Vec<u32> = EcLevel::ALL
            .into_iter()
            .flat_map(|ec| (0..8).map(move |mask| format_bits(ec, mask)))
            .collect();

        assert_eq!(all.len(), 32);
        for (index, a) in all.iter().enumerate() {
            assert!(a >> 15 == 0, "format string is wider than 15 bits");
            for b in &all[index + 1..] {
                assert!(distance(*a, *b) >= 7, "{a:015b} vs {b:015b}");
            }
        }
    }

    #[test]
    fn every_version_string_is_eight_bits_from_every_other() {
        let all: Vec<u32> = (7..=40u8).map(version_bits).collect();

        assert_eq!(all.len(), 34);
        for (index, a) in all.iter().enumerate() {
            assert!(a >> 18 == 0, "version string is wider than 18 bits");
            for b in &all[index + 1..] {
                assert!(distance(*a, *b) >= 8, "{a:018b} vs {b:018b}");
            }
        }
    }

    // The two copies run in opposite directions, so they must be pinned to concrete module
    // patterns. Comparing the copies against each other cannot catch a reversal: it flips both.
    #[test]
    fn the_two_format_copies_run_in_opposite_directions() {
        use crate::quirpy_encoder::matrix;

        assert_eq!(format_bits(EcLevel::M, 2), 0b101_1110_0111_1100);
        let expected = "101111001111100";

        let mut modules = matrix::build(2);
        place_format(&mut modules, EcLevel::M, 2);
        let last = modules.size - 1;
        let read = |cells: &[(usize, usize)]| -> String {
            cells
                .iter()
                .map(|(row, col)| if modules.dark[*row][*col] { '1' } else { '0' })
                .collect()
        };

        let split: Vec<(usize, usize)> = (0..7)
            .map(|index| (last - index, 8))
            .chain((0..8).map(|index| (8, last - 7 + index)))
            .collect();

        assert_eq!(read(&TOP_LEFT), expected.chars().rev().collect::<String>());
        assert_eq!(read(&split), expected);
    }

    #[test]
    fn version_information_lands_in_both_reserved_blocks() {
        use crate::quirpy_encoder::matrix;

        for version in 7..=40u8 {
            let mut modules = matrix::build(version);
            place_version(&mut modules, version);

            let bits = version_bits(version);
            let last = modules.size - 1;

            for index in 0..18usize {
                let want = bits >> index & 1 == 1;
                let (row, col) = (index / 3, index % 3);
                assert_eq!(
                    modules.dark[last - 10 + col][row],
                    want,
                    "v{version} {index}"
                );
                assert_eq!(
                    modules.dark[row][last - 10 + col],
                    want,
                    "v{version} {index}"
                );
            }
        }
    }
}
