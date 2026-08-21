use super::matrix::Modules;

pub const COUNT: u8 = 8;

const FINDER_RUN: [bool; 11] = [
    true, false, true, true, true, false, true, false, false, false, false,
];

pub fn condition(mask: u8, row: usize, col: usize) -> bool {
    let (i, j) = (row, col);
    match mask {
        0 => (i + j) % 2 == 0,
        1 => i % 2 == 0,
        2 => j % 3 == 0,
        3 => (i + j) % 3 == 0,
        4 => (i / 2 + j / 3) % 2 == 0,
        5 => (i * j) % 2 + (i * j) % 3 == 0,
        6 => ((i * j) % 2 + (i * j) % 3) % 2 == 0,
        _ => ((i + j) % 2 + (i * j) % 3) % 2 == 0,
    }
}

pub fn apply(modules: &mut Modules, mask: u8) {
    for row in 0..modules.size {
        for col in 0..modules.size {
            if !modules.reserved[row][col] && condition(mask, row, col) {
                modules.dark[row][col] = !modules.dark[row][col];
            }
        }
    }
}

pub fn penalty(dark: &[Vec<bool>]) -> u32 {
    adjacent_runs(dark) + blocks(dark) + finder_lookalikes(dark) + balance(dark)
}

fn lines(dark: &[Vec<bool>]) -> impl Iterator<Item = Vec<bool>> + '_ {
    let width = dark.first().map_or(0, Vec::len);
    let rows = dark.iter().cloned();
    let columns = (0..width).map(move |col| dark.iter().map(|row| row[col]).collect());
    rows.chain(columns)
}

fn adjacent_runs(dark: &[Vec<bool>]) -> u32 {
    let mut score = 0;
    for line in lines(dark) {
        let mut run = 0;
        let mut previous = None;
        for module in line.iter().copied().chain(std::iter::once(!line[0])) {
            if Some(module) == previous {
                run += 1;
            } else {
                if run >= 5 {
                    score += 3 + (run - 5);
                }
                run = 1;
                previous = Some(module);
            }
        }
    }
    score
}

fn blocks(dark: &[Vec<bool>]) -> u32 {
    let width = dark.first().map_or(0, Vec::len);
    let mut score = 0;
    for row in 0..dark.len() - 1 {
        for col in 0..width - 1 {
            let corner = dark[row][col];
            if dark[row][col + 1] == corner
                && dark[row + 1][col] == corner
                && dark[row + 1][col + 1] == corner
            {
                score += 3;
            }
        }
    }
    score
}

fn finder_lookalikes(dark: &[Vec<bool>]) -> u32 {
    let mut score = 0;
    for line in lines(dark) {
        for window in line.windows(FINDER_RUN.len()) {
            let forward = window.iter().zip(FINDER_RUN).all(|(a, b)| *a == b);
            let backward = window.iter().rev().zip(FINDER_RUN).all(|(a, b)| *a == b);
            if forward || backward {
                score += 40;
            }
        }
    }
    score
}

fn balance(dark: &[Vec<bool>]) -> u32 {
    let total = (dark.len() * dark.first().map_or(0, Vec::len)) as u32;
    let on = dark.iter().flatten().filter(|module| **module).count() as u32;
    10 * ((on * 100).abs_diff(total * 50) / (5 * total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quirpy_encoder::matrix;

    fn grid(rows: &[&str]) -> Vec<Vec<bool>> {
        rows.iter()
            .map(|row| row.chars().map(|c| c == '#').collect())
            .collect()
    }

    #[test]
    fn mask_conditions_match_hand_computed_grids() {
        let expected = [
            // 0: (i + j) % 2 == 0
            [
                "#.#.#.#.", ".#.#.#.#", "#.#.#.#.", ".#.#.#.#", "#.#.#.#.", ".#.#.#.#", "#.#.#.#.",
                ".#.#.#.#",
            ],
            // 1: i % 2 == 0
            [
                "########", "........", "########", "........", "########", "........", "########",
                "........",
            ],
            // 2: j % 3 == 0
            [
                "#..#..#.", "#..#..#.", "#..#..#.", "#..#..#.", "#..#..#.", "#..#..#.", "#..#..#.",
                "#..#..#.",
            ],
            // 3: (i + j) % 3 == 0
            [
                "#..#..#.", "..#..#..", ".#..#..#", "#..#..#.", "..#..#..", ".#..#..#", "#..#..#.",
                "..#..#..",
            ],
            // 4: (i / 2 + j / 3) % 2 == 0
            [
                "###...##", "###...##", "...###..", "...###..", "###...##", "###...##", "...###..",
                "...###..",
            ],
            // 5: (i * j) % 2 + (i * j) % 3 == 0
            [
                "########", "#.....#.", "#..#..#.", "#.#.#.#.", "#..#..#.", "#.....#.", "########",
                "#.....#.",
            ],
            // 6: ((i * j) % 2 + (i * j) % 3) % 2 == 0
            [
                "########", "###...##", "##.##.##", "#.#.#.#.", "#.##.##.", "#...###.", "########",
                "###...##",
            ],
            // 7: ((i + j) % 2 + (i * j) % 3) % 2 == 0
            [
                "#.#.#.#.", "...###..", "#...###.", ".#.#.#.#", "###...##", ".###...#", "#.#.#.#.",
                "...###..",
            ],
        ];

        for (mask, rows) in expected.into_iter().enumerate() {
            for (row, cells) in grid(&rows).into_iter().enumerate() {
                for (col, want) in cells.into_iter().enumerate() {
                    assert_eq!(
                        condition(mask as u8, row, col),
                        want,
                        "mask {mask} at ({row}, {col})"
                    );
                }
            }
        }
    }

    #[test]
    fn a_run_of_seven_scores_five() {
        assert_eq!(adjacent_runs(&grid(&["#######"])), 5);
    }

    #[test]
    fn balance_scores_zero_at_half_dark_and_ten_at_forty_five_percent() {
        let even = grid(&["#.", ".#"]);
        assert_eq!(balance(&even), 0);

        let mut skewed = vec![vec![false; 10]; 10];
        for (index, module) in skewed.iter_mut().flatten().enumerate() {
            *module = index < 45;
        }
        assert_eq!(balance(&skewed), 10);
    }

    #[test]
    fn a_lone_two_by_two_block_scores_three() {
        assert_eq!(blocks(&grid(&["##.", "##.", "..."])), 3);
    }

    #[test]
    fn the_finder_lookalike_is_scored_in_both_directions() {
        assert_eq!(finder_lookalikes(&grid(&["#.###.#...."])), 40);
        assert_eq!(finder_lookalikes(&grid(&["....#.###.#"])), 40);
    }

    #[test]
    fn masking_is_an_involution() {
        for mask in 0..COUNT {
            let mut modules = matrix::build(4);
            modules.dark[10][10] = true;
            let original = modules.clone();

            apply(&mut modules, mask);
            assert_ne!(modules, original, "mask {mask} changed nothing");
            apply(&mut modules, mask);
            assert_eq!(modules, original, "mask {mask}");
        }
    }
}
