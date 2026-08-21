use crate::quirpy_payload::{PayloadError, escape::escape_vcard};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct VCardFields {
    pub first_name: String,
    pub last_name: String,
    pub org: String,
    pub title: String,
    pub phone: String,
    pub email: String,
    pub url: String,
    pub address: String,
}

pub fn build(fields: &VCardFields) -> Result<String, PayloadError> {
    let first = fields.first_name.trim();
    let last = fields.last_name.trim();
    if first.is_empty() && last.is_empty() {
        return Err(PayloadError::MissingField("Name"));
    }

    let full_name = format!("{first} {last}");
    let full_name = full_name.trim();

    let mut lines = vec![
        "BEGIN:VCARD".to_owned(),
        "VERSION:3.0".to_owned(),
        format!("N:{};{};;;", escape_vcard(last), escape_vcard(first)),
        format!("FN:{}", escape_vcard(full_name)),
    ];

    let mut push_if_set = |prefix: &str, value: &str, suffix: &str| {
        let value = value.trim();
        if !value.is_empty() {
            lines.push(format!("{prefix}{}{suffix}", escape_vcard(value)));
        }
    };

    push_if_set("ORG:", &fields.org, "");
    push_if_set("TITLE:", &fields.title, "");
    push_if_set("TEL;TYPE=CELL:", &fields.phone, "");
    push_if_set("EMAIL;TYPE=INTERNET:", &fields.email, "");
    push_if_set("URL:", &fields.url, "");
    push_if_set("ADR;TYPE=HOME:;;", &fields.address, ";;;;");

    lines.push("END:VCARD".to_owned());
    Ok(lines.join("\r\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_populated_contact() {
        let fields = VCardFields {
            first_name: "Ada".to_owned(),
            last_name: "Lovelace".to_owned(),
            org: "Analytical Engines".to_owned(),
            title: "Mathematician".to_owned(),
            phone: "+31612345678".to_owned(),
            email: "ada@example.com".to_owned(),
            url: "https://example.com".to_owned(),
            address: "12 Baker Street, London".to_owned(),
        };

        assert_eq!(
            build(&fields).unwrap(),
            [
                "BEGIN:VCARD",
                "VERSION:3.0",
                "N:Lovelace;Ada;;;",
                "FN:Ada Lovelace",
                "ORG:Analytical Engines",
                "TITLE:Mathematician",
                "TEL;TYPE=CELL:+31612345678",
                "EMAIL;TYPE=INTERNET:ada@example.com",
                "URL:https://example.com",
                r"ADR;TYPE=HOME:;;12 Baker Street\, London;;;;",
                "END:VCARD",
            ]
            .join("\r\n")
        );
    }

    #[test]
    fn optional_lines_are_omitted() {
        let fields = VCardFields {
            first_name: "Ada".to_owned(),
            ..Default::default()
        };

        assert_eq!(
            build(&fields).unwrap(),
            [
                "BEGIN:VCARD",
                "VERSION:3.0",
                "N:;Ada;;;",
                "FN:Ada",
                "END:VCARD"
            ]
            .join("\r\n")
        );
    }

    #[test]
    fn missing_name_is_an_error() {
        let fields = VCardFields {
            org: "Analytical Engines".to_owned(),
            ..Default::default()
        };
        assert_eq!(build(&fields), Err(PayloadError::MissingField("Name")));
    }
}
