pub fn escape_wifi(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if matches!(c, '\\' | ';' | ',' | ':' | '"') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

pub fn escape_vcard(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' | ';' | ',' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

pub fn escape_ical(value: &str) -> String {
    escape_vcard(value)
}

pub fn percent_encode(value: &str) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_escapes_reserved_characters() {
        assert_eq!(escape_wifi(r#"a;b\c,d:e"f"#), r#"a\;b\\c\,d\:e\"f"#);
        assert_eq!(escape_wifi("plain"), "plain");
    }

    #[test]
    fn wifi_escapes_ssid_with_semicolon_and_backslash() {
        assert_eq!(escape_wifi(r"Guest;Net\2"), r"Guest\;Net\\2");
    }

    #[test]
    fn vcard_escapes_and_folds_newlines() {
        assert_eq!(escape_vcard(r"a,b;c\d"), r"a\,b\;c\\d");
        assert_eq!(escape_vcard("line1\r\nline2"), "line1\\nline2");
    }

    #[test]
    fn ical_matches_vcard_rules() {
        let input = "Room 1, floor 2;\nbring badge";
        assert_eq!(escape_ical(input), escape_vcard(input));
    }

    #[test]
    fn percent_encodes_spaces_and_non_ascii() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(percent_encode("caf\u{e9}"), "caf%C3%A9");
        assert_eq!(percent_encode("a-b._~9"), "a-b._~9");
    }
}
