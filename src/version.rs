pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_SHA: &str = env!("QUIRPY_GIT_SHA");
pub const BUILD_DATE: &str = env!("QUIRPY_BUILD_DATE");

const UNKNOWN: &str = "unknown";

pub fn full_version() -> String {
    compose(VERSION, GIT_SHA)
}

fn compose(version: &str, sha: &str) -> String {
    if sha == UNKNOWN || sha.is_empty() {
        version.to_owned()
    } else {
        format!("{version}-{sha}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_sha_becomes_the_build_suffix() {
        assert_eq!(compose("0.0.1", "785e0b7"), "0.0.1-785e0b7");
    }

    #[test]
    fn an_unknown_sha_leaves_a_plain_version() {
        assert_eq!(compose("0.0.1", UNKNOWN), "0.0.1");
        assert_eq!(compose("0.0.1", ""), "0.0.1");
    }

    #[test]
    fn the_baked_build_date_is_a_calendar_date() {
        let parts: Vec<&str> = BUILD_DATE.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|part| part.chars().all(|c| c.is_ascii_digit())));
    }
}
