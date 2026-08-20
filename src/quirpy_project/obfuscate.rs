//! Obfuscation — **not** encryption. The key below is a constant in an open-source
//! repository, so anyone can reverse this in seconds. It exists only to keep `.qpy`
//! contents from being casually read or hand-edited, and must never be described to
//! users as protecting or securing their data.

use base64::prelude::{BASE64_STANDARD, Engine as _};

// Deliberately not a readable phrase: a key that starts with a likely plaintext (the app name,
// say) XORs that plaintext to a run of zero bytes, which is plainly visible in the encoded output.
const KEY: &[u8] = &[
    0x5B, 0xE4, 0x91, 0x2C, 0x7A, 0xD3, 0x08, 0xBF, 0x46, 0x1D, 0xA7, 0x62, 0xCE, 0x39, 0x85, 0xF0,
    0x14, 0x9B, 0x27, 0xD6, 0x6A, 0xB3, 0x4F, 0xE8, 0x71, 0x0C, 0xA2, 0x5D, 0xC9, 0x36, 0x8E, 0xFB,
];

pub fn obfuscate(value: &str) -> String {
    BASE64_STANDARD.encode(xor(value.as_bytes()))
}

pub fn deobfuscate(value: &str) -> Option<String> {
    let bytes = BASE64_STANDARD.decode(value).ok()?;
    String::from_utf8(xor(&bytes)).ok()
}

fn xor(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .zip(KEY.iter().cycle())
        .map(|(byte, key)| byte ^ key)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for value in ["", "hello", "a=b;c\nd", "wachtwoord — ünïcodé 🔑"] {
            assert_eq!(deobfuscate(&obfuscate(value)).as_deref(), Some(value));
        }
    }

    #[test]
    fn output_is_not_the_input() {
        for value in ["secret", "Quirpy"] {
            assert_ne!(obfuscate(value), value);
        }
    }

    // A key beginning with a plausible plaintext would encode that plaintext as zero bytes, which
    // shows up in the file as an obvious run of "A"s.
    #[test]
    fn common_plaintext_does_not_encode_to_zero_bytes() {
        for value in ["Quirpy", "quirpy", "Untitled"] {
            assert!(!xor(value.as_bytes()).iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn garbage_does_not_panic() {
        assert_eq!(deobfuscate("not base64!!"), None);
    }
}
