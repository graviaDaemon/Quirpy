/// Placeholder for the hand-rolled QR encoder. Returns a fixed pattern, ignores input.
/// Will be replaced by real Model 2 encoding logic in a later changeset.
pub fn placeholder_matrix(size: usize) -> Vec<Vec<bool>> {
    let mut matrix = vec![vec![false; size]; size];

    let finder_size = (size / 4).clamp(3, 7);
    draw_finder_pattern(&mut matrix, 0, 0, finder_size);
    if size > finder_size {
        draw_finder_pattern(&mut matrix, size - finder_size, 0, finder_size);
        draw_finder_pattern(&mut matrix, 0, size - finder_size, finder_size);
    }

    for row in matrix.iter_mut().take(size).skip(finder_size) {
        let row_len = row.len();
        for (col, cell) in row.iter_mut().enumerate().take(size).skip(finder_size) {
            *cell = (col + row_len) % 3 == 0;
        }
    }

    matrix
}

fn draw_finder_pattern(matrix: &mut [Vec<bool>], top: usize, left: usize, size: usize) {
    for r in 0..size {
        for c in 0..size {
            let on_border = r == 0 || r == size - 1 || c == 0 || c == size - 1;
            let in_core =
                r >= 2 && r < size.saturating_sub(2) && c >= 2 && c < size.saturating_sub(2);
            matrix[top + r][left + c] = on_border || in_core;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_requested_dimensions_and_is_deterministic() {
        let size = 21;
        let first = placeholder_matrix(size);
        let second = placeholder_matrix(size);

        assert_eq!(first.len(), size);
        assert!(first.iter().all(|row| row.len() == size));
        assert_eq!(first, second);
    }
}
