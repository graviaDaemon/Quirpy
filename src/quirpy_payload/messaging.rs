use crate::quirpy_payload::{PayloadError, escape::percent_encode};
use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MessagingMode {
    #[default]
    Email,
    Sms,
    WhatsApp,
}

impl MessagingMode {
    pub const ALL: [MessagingMode; 3] = [
        MessagingMode::Email,
        MessagingMode::Sms,
        MessagingMode::WhatsApp,
    ];
}

impl fmt::Display for MessagingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            MessagingMode::Email => "Email",
            MessagingMode::Sms => "SMS",
            MessagingMode::WhatsApp => "WhatsApp",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EmailFields {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SmsFields {
    pub number: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WhatsAppFields {
    pub number: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MessagingFields {
    pub mode: MessagingMode,
    pub email: EmailFields,
    pub sms: SmsFields,
    pub whatsapp: WhatsAppFields,
}

pub fn build(fields: &MessagingFields) -> Result<String, PayloadError> {
    match fields.mode {
        MessagingMode::Email => build_email(&fields.email),
        MessagingMode::Sms => build_sms(&fields.sms),
        MessagingMode::WhatsApp => build_whatsapp(&fields.whatsapp),
    }
}

fn build_email(fields: &EmailFields) -> Result<String, PayloadError> {
    let to = fields.to.trim();
    if to.is_empty() {
        return Err(PayloadError::MissingField("Recipient"));
    }

    let mut query = Vec::new();
    if !fields.subject.is_empty() {
        query.push(format!("subject={}", percent_encode(&fields.subject)));
    }
    if !fields.body.is_empty() {
        query.push(format!("body={}", percent_encode(&fields.body)));
    }

    if query.is_empty() {
        Ok(format!("mailto:{to}"))
    } else {
        Ok(format!("mailto:{to}?{}", query.join("&")))
    }
}

fn build_sms(fields: &SmsFields) -> Result<String, PayloadError> {
    let number = fields.number.trim();
    if number.is_empty() {
        return Err(PayloadError::MissingField("Phone number"));
    }
    Ok(format!("SMSTO:{number}:{}", fields.message))
}

fn build_whatsapp(fields: &WhatsAppFields) -> Result<String, PayloadError> {
    let digits: String = fields.number.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return Err(PayloadError::Invalid {
            field: "Phone number",
            reason: "must contain digits, including the country code".to_owned(),
        });
    }

    if fields.text.is_empty() {
        Ok(format!("https://wa.me/{digits}"))
    } else {
        Ok(format!(
            "https://wa.me/{digits}?text={}",
            percent_encode(&fields.text)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_with_subject_and_body() {
        let fields = EmailFields {
            to: "ada@example.com".to_owned(),
            subject: "Hello there".to_owned(),
            body: "Line one\nLine two".to_owned(),
        };
        assert_eq!(
            build_email(&fields).unwrap(),
            "mailto:ada@example.com?subject=Hello%20there&body=Line%20one%0ALine%20two"
        );
    }

    #[test]
    fn email_without_subject_or_body_has_no_query() {
        let fields = EmailFields {
            to: "ada@example.com".to_owned(),
            ..Default::default()
        };
        assert_eq!(build_email(&fields).unwrap(), "mailto:ada@example.com");
    }

    #[test]
    fn email_requires_a_recipient() {
        assert_eq!(
            build_email(&EmailFields::default()),
            Err(PayloadError::MissingField("Recipient"))
        );
    }

    #[test]
    fn sms_uses_smsto_form() {
        let fields = SmsFields {
            number: "+31612345678".to_owned(),
            message: "On my way".to_owned(),
        };
        assert_eq!(
            build_sms(&fields).unwrap(),
            "SMSTO:+31612345678:On my way"
        );
    }

    #[test]
    fn sms_without_message_keeps_the_trailing_colon() {
        let fields = SmsFields {
            number: "0612345678".to_owned(),
            ..Default::default()
        };
        assert_eq!(build_sms(&fields).unwrap(), "SMSTO:0612345678:");
    }

    #[test]
    fn sms_requires_a_number() {
        assert_eq!(
            build_sms(&SmsFields::default()),
            Err(PayloadError::MissingField("Phone number"))
        );
    }

    #[test]
    fn whatsapp_strips_formatting_from_the_number() {
        let fields = WhatsAppFields {
            number: "+31 (6) 12-345-678".to_owned(),
            text: "Hi!".to_owned(),
        };
        assert_eq!(
            build_whatsapp(&fields).unwrap(),
            "https://wa.me/31612345678?text=Hi%21"
        );
    }

    #[test]
    fn whatsapp_without_text_omits_the_query() {
        let fields = WhatsAppFields {
            number: "31612345678".to_owned(),
            ..Default::default()
        };
        assert_eq!(build_whatsapp(&fields).unwrap(), "https://wa.me/31612345678");
    }

    #[test]
    fn whatsapp_requires_digits() {
        let fields = WhatsAppFields {
            number: "+-()".to_owned(),
            ..Default::default()
        };
        assert!(matches!(
            build_whatsapp(&fields),
            Err(PayloadError::Invalid { .. })
        ));
    }

    #[test]
    fn mode_selects_the_builder() {
        let fields = MessagingFields {
            mode: MessagingMode::Sms,
            email: EmailFields {
                to: "ada@example.com".to_owned(),
                ..Default::default()
            },
            sms: SmsFields {
                number: "0612345678".to_owned(),
                message: "hi".to_owned(),
            },
            whatsapp: WhatsAppFields::default(),
        };
        assert_eq!(build(&fields).unwrap(), "SMSTO:0612345678:hi");
    }
}
