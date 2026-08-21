use std::sync::LazyLock;

const PRIMITIVE: u16 = 0x11D;

struct Tables {
    exp: [u8; 512],
    log: [u8; 256],
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| {
    let mut exp = [0u8; 512];
    let mut log = [0u8; 256];

    let mut value: u16 = 1;
    for (power, slot) in exp.iter_mut().take(255).enumerate() {
        *slot = value as u8;
        log[value as usize] = power as u8;
        value <<= 1;
        if value & 0x100 != 0 {
            value ^= PRIMITIVE;
        }
    }
    for i in 255..exp.len() {
        exp[i] = exp[i - 255];
    }

    Tables { exp, log }
});

pub fn mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let tables = &*TABLES;
    tables.exp[tables.log[a as usize] as usize + tables.log[b as usize] as usize]
}

pub fn generator_poly(degree: usize) -> Vec<u8> {
    let tables = &*TABLES;
    let mut poly = vec![1u8];

    for i in 0..degree {
        let root = tables.exp[i];
        poly.push(0);
        for index in (1..poly.len()).rev() {
            poly[index] ^= mul(poly[index - 1], root);
        }
    }

    poly
}

pub fn remainder(data: &[u8], generator: &[u8]) -> Vec<u8> {
    let ec_len = generator.len() - 1;
    let mut buffer = data.to_vec();
    buffer.resize(data.len() + ec_len, 0);

    for step in 0..data.len() {
        let lead = buffer[step];
        if lead == 0 {
            continue;
        }
        for (offset, &coefficient) in generator.iter().enumerate() {
            buffer[step + offset] ^= mul(coefficient, lead);
        }
    }

    buffer.split_off(data.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_and_log_round_trip() {
        let tables = &*TABLES;
        for value in 1..=255u8 {
            assert_eq!(tables.exp[tables.log[value as usize] as usize], value);
        }
    }

    #[test]
    fn multiplication_is_commutative_with_one_as_identity() {
        for a in 0..=255u8 {
            assert_eq!(mul(a, 1), a);
            assert_eq!(mul(1, a), a);
            for b in 0..=255u8 {
                assert_eq!(mul(a, b), mul(b, a));
            }
        }
    }

    #[test]
    fn zero_annihilates() {
        for a in 0..=255u8 {
            assert_eq!(mul(a, 0), 0);
            assert_eq!(mul(0, a), 0);
        }
    }

    #[test]
    fn degree_seven_generator_matches_the_spec() {
        assert_eq!(generator_poly(7), vec![1, 127, 122, 154, 164, 11, 68, 117]);
    }

    // ISO/IEC 18004 Table A.1 lists the generator polynomials as powers of alpha.
    #[test]
    fn generator_exponents_match_the_spec_table() {
        let tables = &*TABLES;
        let exponents = |degree| -> Vec<u8> {
            generator_poly(degree)
                .into_iter()
                .map(|coefficient| tables.log[coefficient as usize])
                .collect()
        };

        assert_eq!(exponents(7), vec![0, 87, 229, 146, 149, 238, 102, 21]);
        assert_eq!(
            exponents(10),
            vec![0, 251, 67, 46, 61, 118, 70, 64, 94, 32, 45]
        );
        assert_eq!(
            exponents(13),
            vec![
                0, 74, 152, 176, 100, 86, 100, 106, 104, 130, 218, 206, 140, 78
            ]
        );
    }

    #[test]
    fn generators_are_monic_and_of_the_requested_degree() {
        for degree in 1..=30 {
            let poly = generator_poly(degree);
            assert_eq!(poly.len(), degree + 1);
            assert_eq!(poly[0], 1);
        }
    }

    // ISO/IEC 18004 Annex I worked example: version 1-M, payload "01234567".
    #[test]
    fn the_spec_worked_example_produces_its_published_ec_codewords() {
        let data = [
            0x10, 0x20, 0x0C, 0x56, 0x61, 0x80, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11,
            0xEC, 0x11,
        ];
        assert_eq!(
            remainder(&data, &generator_poly(10)),
            vec![0xA5, 0x24, 0xD4, 0xC1, 0xED, 0x36, 0xC7, 0x87, 0x2C, 0x55]
        );
    }
}
