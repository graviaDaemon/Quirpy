use crate::quirpy_payload::{PayloadError, escape::percent_encode};
use std::fmt;

pub const DEFAULT_DIGITS: u8 = 6;
pub const DEFAULT_PERIOD: u32 = 30;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Algorithm {
    #[default]
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
    pub const ALL: [Algorithm; 3] = [Algorithm::Sha1, Algorithm::Sha256, Algorithm::Sha512];
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Algorithm::Sha1 => "SHA1",
            Algorithm::Sha256 => "SHA256",
            Algorithm::Sha512 => "SHA512",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MfaFields {
    pub issuer: String,
    pub account: String,
    pub secret: String,
    pub algorithm: Algorithm,
    pub digits: u8,
    pub period: u32,
}

impl Default for MfaFields {
    fn default() -> Self {
        Self {
            issuer: String::new(),
            account: String::new(),
            secret: String::new(),
            algorithm: Algorithm::default(),
            digits: DEFAULT_DIGITS,
            period: DEFAULT_PERIOD,
        }
    }
}

fn normalise_secret(secret: &str) -> Result<String, PayloadError> {
    let cleaned: String = secret
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();

    if cleaned.is_empty() {
        return Err(PayloadError::MissingField("Secret"));
    }

    let body = cleaned.trim_end_matches('=');
    let is_base32 = !body.is_empty()
        && body
            .chars()
            .all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c));

    if !is_base32 {
        return Err(PayloadError::Invalid {
            field: "Secret",
            reason: "must be base32 (A-Z, 2-7)".to_owned(),
        });
    }

    Ok(cleaned)
}

pub fn build(fields: &MfaFields) -> Result<String, PayloadError> {
    let account = fields.account.trim();
    if account.is_empty() {
        return Err(PayloadError::MissingField("Account"));
    }
    let secret = normalise_secret(&fields.secret)?;
    let issuer = fields.issuer.trim();

    let label = if issuer.is_empty() {
        percent_encode(account)
    } else {
        format!("{}:{}", percent_encode(issuer), percent_encode(account))
    };

    let mut out = format!("otpauth://totp/{label}?secret={secret}");
    if !issuer.is_empty() {
        out.push_str(&format!("&issuer={}", percent_encode(issuer)));
    }
    if fields.algorithm != Algorithm::default() {
        out.push_str(&format!("&algorithm={}", fields.algorithm));
    }
    if fields.digits != DEFAULT_DIGITS {
        out.push_str(&format!("&digits={}", fields.digits));
    }
    if fields.period != DEFAULT_PERIOD {
        out.push_str(&format!("&period={}", fields.period));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_default_parameters_are_appended() {
        let fields = MfaFields {
            issuer: "Quirpy Ltd".to_owned(),
            account: "ada@example.com".to_owned(),
            secret: "jbsw y3dp ehpk 3pxp".to_owned(),
            algorithm: Algorithm::Sha256,
            digits: 8,
            period: 60,
        };
        assert_eq!(
            build(&fields).unwrap(),
            "otpauth://totp/Quirpy%20Ltd:ada%40example.com?secret=JBSWY3DPEHPK3PXP\
             &issuer=Quirpy%20Ltd&algorithm=SHA256&digits=8&period=60"
        );
    }

    #[test]
    fn default_parameters_are_omitted() {
        let fields = MfaFields {
            issuer: "Quirpy".to_owned(),
            account: "ada".to_owned(),
            secret: "JBSWY3DPEHPK3PXP".to_owned(),
            ..Default::default()
        };
        assert_eq!(
            build(&fields).unwrap(),
            "otpauth://totp/Quirpy:ada?secret=JBSWY3DPEHPK3PXP&issuer=Quirpy"
        );
    }

    #[test]
    fn issuer_is_optional() {
        let fields = MfaFields {
            account: "ada".to_owned(),
            secret: "JBSWY3DPEHPK3PXP".to_owned(),
            ..Default::default()
        };
        assert_eq!(
            build(&fields).unwrap(),
            "otpauth://totp/ada?secret=JBSWY3DPEHPK3PXP"
        );
    }

    #[test]
    fn padding_is_accepted() {
        assert_eq!(normalise_secret("jbswy3dp===").unwrap(), "JBSWY3DP===");
    }

    #[test]
    fn missing_account_is_an_error() {
        let fields = MfaFields {
            secret: "JBSWY3DPEHPK3PXP".to_owned(),
            ..Default::default()
        };
        assert_eq!(build(&fields), Err(PayloadError::MissingField("Account")));
    }

    #[test]
    fn non_base32_secret_is_an_error() {
        let fields = MfaFields {
            account: "ada".to_owned(),
            secret: "18".to_owned(),
            ..Default::default()
        };
        assert!(matches!(build(&fields), Err(PayloadError::Invalid { .. })));
    }

    #[test]
    fn empty_secret_is_an_error() {
        let fields = MfaFields {
            account: "ada".to_owned(),
            ..Default::default()
        };
        assert_eq!(build(&fields), Err(PayloadError::MissingField("Secret")));
    }
}
