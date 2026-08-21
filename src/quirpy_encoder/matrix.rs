use super::tables::ALIGNMENT_CENTRES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modules {
    pub size: usize,
    pub dark: Vec<Vec<bool>>,
    pub reserved: Vec<Vec<bool>>,
}

pub fn size(version: u8) -> usize {
    4 * version as usize + 17
}

impl Modules {
    fn new(version: u8) -> Self {
        let size = size(version);
        Self {
            size,
            dark: vec![vec![false; size]; size],
            reserved: vec![vec![false; size]; size],
        }
    }

    fn reserve(&mut self, top: usize, left: usize, height: usize, width: usize) {
        for row in top..top + height {
            for cell in self.reserved[row][left..left + width].iter_mut() {
                *cell = true;
            }
        }
    }
}

pub fn build(version: u8) -> Modules {
    let mut modules = Modules::new(version);
    let size = modules.size;

    for (top, left) in [(0, 0), (0, size - 7), (size - 7, 0)] {
        finder(&mut modules, top, left);
    }
    modules.reserve(0, 0, 8, 8);
    modules.reserve(0, size - 8, 8, 8);
    modules.reserve(size - 8, 0, 8, 8);

    for index in 8..size - 8 {
        let on = index % 2 == 0;
        modules.dark[6][index] = on;
        modules.dark[index][6] = on;
        modules.reserved[6][index] = true;
        modules.reserved[index][6] = true;
    }

    alignment(&mut modules, version);

    modules.dark[size - 8][8] = true;

    for index in 0..9 {
        modules.reserved[8][index] = true;
        modules.reserved[index][8] = true;
    }
    for index in size - 8..size {
        modules.reserved[8][index] = true;
        modules.reserved[index][8] = true;
    }

    if version >= 7 {
        modules.reserve(size - 11, 0, 3, 6);
        modules.reserve(0, size - 11, 6, 3);
    }

    modules
}

fn finder(modules: &mut Modules, top: usize, left: usize) {
    for row in 0..7usize {
        for col in 0..7usize {
            let ring = row.abs_diff(3).max(col.abs_diff(3));
            modules.dark[top + row][left + col] = ring != 2;
        }
    }
}

fn alignment(modules: &mut Modules, version: u8) {
    let centres = ALIGNMENT_CENTRES[version as usize - 1];
    let Some((&first, &last)) = centres.first().zip(centres.last()) else {
        return;
    };

    for &row in centres {
        for &col in centres {
            let collides = (row, col) == (first, first)
                || (row, col) == (first, last)
                || (row, col) == (last, first);
            if collides {
                continue;
            }

            let (row, col) = (row as usize, col as usize);
            for dr in 0..5usize {
                for dc in 0..5usize {
                    let ring = dr.abs_diff(2).max(dc.abs_diff(2));
                    modules.dark[row - 2 + dr][col - 2 + dc] = ring != 1;
                    modules.reserved[row - 2 + dr][col - 2 + dc] = true;
                }
            }
        }
    }
}

pub fn free_modules(version: u8) -> usize {
    build(version)
        .reserved
        .iter()
        .flatten()
        .filter(|reserved| !**reserved)
        .count()
}

pub fn place_data(modules: &mut Modules, stream: &[bool]) {
    let size = modules.size;
    let mut bits = stream.iter();
    let mut upward = true;
    let mut right = size - 1;

    loop {
        if right == 6 {
            right -= 1;
        }

        for step in 0..size {
            let row = if upward { size - 1 - step } else { step };
            for col in [right, right - 1] {
                if modules.reserved[row][col] {
                    continue;
                }
                modules.dark[row][col] = bits.next().copied().unwrap_or(false);
            }
        }

        upward = !upward;
        if right < 3 {
            break;
        }
        right -= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_size_grows_by_four_per_version() {
        assert_eq!(size(1), 21);
        assert_eq!(size(40), 177);
    }

    fn alignment_count(version: u8) -> usize {
        let centres = ALIGNMENT_CENTRES[version as usize - 1];
        if centres.is_empty() {
            return 0;
        }
        centres.len() * centres.len() - 3
    }

    #[test]
    fn alignment_pattern_counts_match_the_spec() {
        assert_eq!(alignment_count(1), 0);
        assert_eq!(alignment_count(7), 6);
        assert_eq!(alignment_count(40), 46);
    }

    #[test]
    fn the_timing_pattern_alternates_from_the_separators_inwards() {
        let modules = build(5);
        for col in 8..modules.size - 8 {
            assert_eq!(modules.dark[6][col], col % 2 == 0, "column {col}");
            assert_eq!(modules.dark[col][6], col % 2 == 0, "row {col}");
        }
    }

    #[test]
    fn the_dark_module_is_always_set() {
        for version in 1..=40u8 {
            let modules = build(version);
            assert!(
                modules.dark[4 * version as usize + 9][8],
                "version {version}"
            );
        }
    }

    #[test]
    fn free_modules_match_the_derived_codeword_count() {
        use crate::quirpy_encoder::tables;

        for version in 1..=40u8 {
            assert_eq!(
                free_modules(version),
                tables::total_codewords(version) * 8 + tables::remainder_bits(version),
                "version {version}"
            );
        }
    }

    #[test]
    fn data_placement_visits_every_free_module_exactly_once() {
        for version in [1u8, 2, 7, 14, 27, 40] {
            let free = free_modules(version);
            let stream: Vec<bool> = (0..free).map(|index| index % 3 == 0).collect();

            let mut modules = build(version);
            place_data(&mut modules, &stream);

            let mut seen = Vec::with_capacity(free);
            for row in 0..modules.size {
                for col in 0..modules.size {
                    if !modules.reserved[row][col] {
                        seen.push(modules.dark[row][col]);
                    }
                }
            }
            assert_eq!(seen.len(), free, "version {version}");
            assert_eq!(
                seen.iter().filter(|bit| **bit).count(),
                stream.iter().filter(|bit| **bit).count(),
                "version {version}"
            );
        }
    }
}
