use crate::quirpy_payload::PayloadError;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UrlFields {
    pub url: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextFields {
    pub text: String,
}

pub fn build_url(fields: &UrlFields) -> Result<String, PayloadError> {
    let url = fields.url.trim();
    if url.is_empty() {
        return Err(PayloadError::MissingField("URL"));
    }
    if url.contains("://") {
        Ok(url.to_owned())
    } else {
        Ok(format!("https://{url}"))
    }
}

pub fn build_text(fields: &TextFields) -> Result<String, PayloadError> {
    if fields.text.trim().is_empty() {
        return Err(PayloadError::MissingField("Text"));
    }
    Ok(fields.text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_is_used_verbatim_when_it_has_a_scheme() {
        let fields = UrlFields {
            url: "https://example.com/a?b=c".to_owned(),
        };
        assert_eq!(build_url(&fields).unwrap(), "https://example.com/a?b=c");
    }

    #[test]
    fn url_without_scheme_gets_https() {
        let fields = UrlFields {
            url: "  example.com  ".to_owned(),
        };
        assert_eq!(build_url(&fields).unwrap(), "https://example.com");
    }

    #[test]
    fn empty_url_is_an_error() {
        assert_eq!(
            build_url(&UrlFields::default()),
            Err(PayloadError::MissingField("URL"))
        );
    }

    #[test]
    fn text_is_verbatim() {
        let fields = TextFields {
            text: "hello\nworld".to_owned(),
        };
        assert_eq!(build_text(&fields).unwrap(), "hello\nworld");
    }

    #[test]
    fn empty_text_is_an_error() {
        assert_eq!(
            build_text(&TextFields::default()),
            Err(PayloadError::MissingField("Text"))
        );
    }
}
