pub mod calendar;
pub mod escape;
pub mod messaging;
pub mod mfa;
pub mod simple;
pub mod vcard;
pub mod wifi;

use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum QrDataType {
    #[default]
    Url,
    Text,
    Wifi,
    VCard,
    Calendar,
    Messaging,
    Mfa,
}

impl QrDataType {
    pub const ALL: [QrDataType; 7] = [
        QrDataType::Url,
        QrDataType::Text,
        QrDataType::Wifi,
        QrDataType::VCard,
        QrDataType::Calendar,
        QrDataType::Messaging,
        QrDataType::Mfa,
    ];
}

impl fmt::Display for QrDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            QrDataType::Url => "URL / Website",
            QrDataType::Text => "Text / Alphanumeric",
            QrDataType::Wifi => "Wi-Fi",
            QrDataType::VCard => "vCard / Contact",
            QrDataType::Calendar => "Event / Calendar",
            QrDataType::Messaging => "Email / SMS / WhatsApp",
            QrDataType::Mfa => "MFA (otpauth)",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DataFields {
    pub url: simple::UrlFields,
    pub text: simple::TextFields,
    pub wifi: wifi::WifiFields,
    pub vcard: vcard::VCardFields,
    pub calendar: calendar::CalendarFields,
    pub messaging: messaging::MessagingFields,
    pub mfa: mfa::MfaFields,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PayloadError {
    MissingField(&'static str),
    Invalid { field: &'static str, reason: String },
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayloadError::MissingField(field) => write!(f, "{field} is required"),
            PayloadError::Invalid { field, reason } => write!(f, "{field} {reason}"),
        }
    }
}

pub fn build(data_type: QrDataType, fields: &DataFields) -> Result<String, PayloadError> {
    match data_type {
        QrDataType::Url => simple::build_url(&fields.url),
        QrDataType::Text => simple::build_text(&fields.text),
        QrDataType::Wifi => wifi::build(&fields.wifi),
        QrDataType::VCard => vcard::build(&fields.vcard),
        QrDataType::Calendar => calendar::build(&fields.calendar),
        QrDataType::Messaging => messaging::build(&fields.messaging),
        QrDataType::Mfa => mfa::build(&fields.mfa),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_selects_the_matching_builder() {
        let mut fields = DataFields::default();
        fields.url.url = "example.com".to_owned();
        fields.text.text = "plain".to_owned();

        assert_eq!(
            build(QrDataType::Url, &fields).unwrap(),
            "https://example.com"
        );
        assert_eq!(build(QrDataType::Text, &fields).unwrap(), "plain");
    }

    #[test]
    fn errors_render_as_user_facing_text() {
        assert_eq!(PayloadError::MissingField("SSID").to_string(), "SSID is required");
        assert_eq!(
            PayloadError::Invalid {
                field: "Secret",
                reason: "must be base32 (A-Z, 2-7)".to_owned(),
            }
            .to_string(),
            "Secret must be base32 (A-Z, 2-7)"
        );
    }
}
