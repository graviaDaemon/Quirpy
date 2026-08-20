mod obfuscate;

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::Path;

use ini::Ini;
use sha2::{Digest, Sha256};

use crate::quirpy_front::form::ProjectState;
use crate::quirpy_front::style::StyleState;
use crate::quirpy_payload::{
    QrDataType, messaging::MessagingMode, mfa::Algorithm, wifi::WifiSecurity,
};

use obfuscate::{deobfuscate, obfuscate};

pub const SCHEMA_VERSION: u32 = 1;

const META: &str = "meta";

#[derive(Debug)]
pub enum ProjectFileError {
    Io(io::Error),
    Parse(String),
    ChecksumMismatch,
    UnsupportedSchema(u32),
}

impl fmt::Display for ProjectFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectFileError::Io(error) => write!(f, "{error}"),
            ProjectFileError::Parse(reason) => {
                write!(f, "This file is not a Quirpy project: {reason}")
            }
            ProjectFileError::ChecksumMismatch => write!(
                f,
                "This file appears to be corrupt or was edited outside Quirpy."
            ),
            ProjectFileError::UnsupportedSchema(version) => write!(
                f,
                "This project was saved by a newer version of Quirpy (schema {version}, this build \
                 understands {SCHEMA_VERSION})."
            ),
        }
    }
}

impl std::error::Error for ProjectFileError {}

impl From<io::Error> for ProjectFileError {
    fn from(error: io::Error) -> Self {
        ProjectFileError::Io(error)
    }
}

pub fn save(project: &ProjectState, path: &Path) -> Result<(), ProjectFileError> {
    let entries: Vec<Entry> = to_entries(project)
        .into_iter()
        .map(|(section, key, value)| (section, key, obfuscate(&value)))
        .collect();

    let mut ini = Ini::new();
    ini.with_section(Some(META))
        .set("app_version", env!("CARGO_PKG_VERSION"))
        .set("schema_version", SCHEMA_VERSION.to_string())
        .set("checksum", checksum(&entries));

    for (section, key, value) in &entries {
        ini.set_to(Some(*section), (*key).to_owned(), value.clone());
    }

    ini.write_to_file(path)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<ProjectState, ProjectFileError> {
    load_with(path, true)
}

pub fn load_ignoring_checksum(path: &Path) -> Result<ProjectState, ProjectFileError> {
    load_with(path, false)
}

fn load_with(path: &Path, verify: bool) -> Result<ProjectState, ProjectFileError> {
    let ini = Ini::load_from_file(path).map_err(|error| match error {
        ini::Error::Io(error) => ProjectFileError::Io(error),
        ini::Error::Parse(error) => ProjectFileError::Parse(error.to_string()),
    })?;

    let meta = ini
        .section(Some(META))
        .ok_or_else(|| ProjectFileError::Parse("no [meta] section".to_owned()))?;

    let schema_version: u32 = meta
        .get("schema_version")
        .ok_or_else(|| ProjectFileError::Parse("no schema_version in [meta]".to_owned()))?
        .trim()
        .parse()
        .map_err(|_| ProjectFileError::Parse("schema_version is not a number".to_owned()))?;

    if schema_version > SCHEMA_VERSION {
        return Err(ProjectFileError::UnsupportedSchema(schema_version));
    }

    let entries: Vec<Entry> = entry_keys()
        .into_iter()
        .map(|(section, key)| {
            let value = ini
                .get_from(Some(section), key)
                .unwrap_or_default()
                .to_owned();
            (section, key, value)
        })
        .collect();

    if verify {
        let expected = meta
            .get("checksum")
            .ok_or_else(|| ProjectFileError::Parse("no checksum in [meta]".to_owned()))?;
        if !checksum(&entries).eq_ignore_ascii_case(expected.trim()) {
            return Err(ProjectFileError::ChecksumMismatch);
        }
    }

    let values: HashMap<(&str, &str), String> = entries
        .iter()
        .filter_map(|(section, key, value)| {
            deobfuscate(value).map(|plain| ((*section, *key), plain))
        })
        .collect();

    Ok(from_values(&Values(values)))
}

type Entry = (&'static str, &'static str, String);

fn checksum(entries: &[Entry]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_body(entries).as_bytes());
    hasher
        .finalize()
        .iter()
        .fold(String::new(), |mut out, byte| {
            use fmt::Write as _;
            let _ = write!(out, "{byte:02x}");
            out
        })
}

fn canonical_body(entries: &[Entry]) -> String {
    let mut body = String::new();
    let mut current = "";
    for (section, key, value) in entries {
        if *section != current {
            body.push_str(&format!("[{section}]\n"));
            current = section;
        }
        body.push_str(&format!("{key}={value}\n"));
    }
    body
}

fn entry_keys() -> Vec<(&'static str, &'static str)> {
    to_entries(&ProjectState::default())
        .into_iter()
        .map(|(section, key, _)| (section, key))
        .collect()
}

fn to_entries(project: &ProjectState) -> Vec<Entry> {
    let fields = &project.fields;
    let wifi = &fields.wifi;
    let vcard = &fields.vcard;
    let calendar = &fields.calendar;
    let messaging = &fields.messaging;
    let mfa = &fields.mfa;

    vec![
        ("project", "name", project.name.clone()),
        (
            "project",
            "data_type",
            data_type_key(project.data_type).to_owned(),
        ),
        ("colors", "dark", hex(project.style.dark)),
        ("colors", "light", hex(project.style.light)),
        ("fields.url", "url", fields.url.url.clone()),
        ("fields.text", "text", fields.text.text.clone()),
        ("fields.wifi", "ssid", wifi.ssid.clone()),
        ("fields.wifi", "password", wifi.password.clone()),
        (
            "fields.wifi",
            "security",
            wifi_security_key(wifi.security).to_owned(),
        ),
        ("fields.wifi", "hidden", wifi.hidden.to_string()),
        ("fields.vcard", "first_name", vcard.first_name.clone()),
        ("fields.vcard", "last_name", vcard.last_name.clone()),
        ("fields.vcard", "org", vcard.org.clone()),
        ("fields.vcard", "title", vcard.title.clone()),
        ("fields.vcard", "phone", vcard.phone.clone()),
        ("fields.vcard", "email", vcard.email.clone()),
        ("fields.vcard", "url", vcard.url.clone()),
        ("fields.vcard", "address", vcard.address.clone()),
        ("fields.calendar", "title", calendar.title.clone()),
        ("fields.calendar", "location", calendar.location.clone()),
        (
            "fields.calendar",
            "description",
            calendar.description.clone(),
        ),
        ("fields.calendar", "start", calendar.start.to_string()),
        ("fields.calendar", "end", calendar.end.to_string()),
        ("fields.calendar", "all_day", calendar.all_day.to_string()),
        (
            "fields.messaging",
            "mode",
            messaging_mode_key(messaging.mode).to_owned(),
        ),
        ("fields.messaging", "email_to", messaging.email.to.clone()),
        (
            "fields.messaging",
            "email_subject",
            messaging.email.subject.clone(),
        ),
        (
            "fields.messaging",
            "email_body",
            messaging.email.body.clone(),
        ),
        (
            "fields.messaging",
            "sms_number",
            messaging.sms.number.clone(),
        ),
        (
            "fields.messaging",
            "sms_message",
            messaging.sms.message.clone(),
        ),
        (
            "fields.messaging",
            "whatsapp_number",
            messaging.whatsapp.number.clone(),
        ),
        (
            "fields.messaging",
            "whatsapp_text",
            messaging.whatsapp.text.clone(),
        ),
        ("fields.mfa", "issuer", mfa.issuer.clone()),
        ("fields.mfa", "account", mfa.account.clone()),
        ("fields.mfa", "secret", mfa.secret.clone()),
        (
            "fields.mfa",
            "algorithm",
            algorithm_key(mfa.algorithm).to_owned(),
        ),
        ("fields.mfa", "digits", mfa.digits.to_string()),
        ("fields.mfa", "period", mfa.period.to_string()),
    ]
}

struct Values(HashMap<(&'static str, &'static str), String>);

impl Values {
    fn text(&self, section: &'static str, key: &'static str) -> Option<String> {
        self.0.get(&(section, key)).cloned()
    }

    fn flag(&self, section: &'static str, key: &'static str) -> Option<bool> {
        self.0.get(&(section, key))?.trim().parse().ok()
    }
}

fn from_values(values: &Values) -> ProjectState {
    let mut project = ProjectState::default();

    if let Some(name) = values.text("project", "name") {
        project.name = name;
    }
    if let Some(data_type) = values
        .text("project", "data_type")
        .and_then(|value| data_type_from_key(&value))
    {
        project.data_type = data_type;
    }

    project.style = StyleState {
        dark: values
            .text("colors", "dark")
            .and_then(|value| from_hex(&value))
            .unwrap_or(project.style.dark),
        light: values
            .text("colors", "light")
            .and_then(|value| from_hex(&value))
            .unwrap_or(project.style.light),
    };

    let fields = &mut project.fields;

    if let Some(url) = values.text("fields.url", "url") {
        fields.url.url = url;
    }
    if let Some(text) = values.text("fields.text", "text") {
        fields.text.text = text;
    }

    if let Some(ssid) = values.text("fields.wifi", "ssid") {
        fields.wifi.ssid = ssid;
    }
    if let Some(password) = values.text("fields.wifi", "password") {
        fields.wifi.password = password;
    }
    if let Some(security) = values
        .text("fields.wifi", "security")
        .and_then(|value| wifi_security_from_key(&value))
    {
        fields.wifi.security = security;
    }
    if let Some(hidden) = values.flag("fields.wifi", "hidden") {
        fields.wifi.hidden = hidden;
    }

    let vcard = &mut fields.vcard;
    if let Some(value) = values.text("fields.vcard", "first_name") {
        vcard.first_name = value;
    }
    if let Some(value) = values.text("fields.vcard", "last_name") {
        vcard.last_name = value;
    }
    if let Some(value) = values.text("fields.vcard", "org") {
        vcard.org = value;
    }
    if let Some(value) = values.text("fields.vcard", "title") {
        vcard.title = value;
    }
    if let Some(value) = values.text("fields.vcard", "phone") {
        vcard.phone = value;
    }
    if let Some(value) = values.text("fields.vcard", "email") {
        vcard.email = value;
    }
    if let Some(value) = values.text("fields.vcard", "url") {
        vcard.url = value;
    }
    if let Some(value) = values.text("fields.vcard", "address") {
        vcard.address = value;
    }

    let calendar = &mut fields.calendar;
    if let Some(value) = values.text("fields.calendar", "title") {
        calendar.title = value;
    }
    if let Some(value) = values.text("fields.calendar", "location") {
        calendar.location = value;
    }
    if let Some(value) = values.text("fields.calendar", "description") {
        calendar.description = value;
    }
    if let Some(value) = values
        .text("fields.calendar", "start")
        .and_then(|value| value.trim().parse().ok())
    {
        calendar.start = value;
    }
    if let Some(value) = values
        .text("fields.calendar", "end")
        .and_then(|value| value.trim().parse().ok())
    {
        calendar.end = value;
    }
    if let Some(value) = values.flag("fields.calendar", "all_day") {
        calendar.all_day = value;
    }

    let messaging = &mut fields.messaging;
    if let Some(value) = values
        .text("fields.messaging", "mode")
        .and_then(|value| messaging_mode_from_key(&value))
    {
        messaging.mode = value;
    }
    if let Some(value) = values.text("fields.messaging", "email_to") {
        messaging.email.to = value;
    }
    if let Some(value) = values.text("fields.messaging", "email_subject") {
        messaging.email.subject = value;
    }
    if let Some(value) = values.text("fields.messaging", "email_body") {
        messaging.email.body = value;
    }
    if let Some(value) = values.text("fields.messaging", "sms_number") {
        messaging.sms.number = value;
    }
    if let Some(value) = values.text("fields.messaging", "sms_message") {
        messaging.sms.message = value;
    }
    if let Some(value) = values.text("fields.messaging", "whatsapp_number") {
        messaging.whatsapp.number = value;
    }
    if let Some(value) = values.text("fields.messaging", "whatsapp_text") {
        messaging.whatsapp.text = value;
    }

    let mfa = &mut fields.mfa;
    if let Some(value) = values.text("fields.mfa", "issuer") {
        mfa.issuer = value;
    }
    if let Some(value) = values.text("fields.mfa", "account") {
        mfa.account = value;
    }
    if let Some(value) = values.text("fields.mfa", "secret") {
        mfa.secret = value;
    }
    if let Some(value) = values
        .text("fields.mfa", "algorithm")
        .and_then(|value| algorithm_from_key(&value))
    {
        mfa.algorithm = value;
    }
    if let Some(value) = values
        .text("fields.mfa", "digits")
        .and_then(|value| value.trim().parse().ok())
    {
        mfa.digits = value;
    }
    if let Some(value) = values
        .text("fields.mfa", "period")
        .and_then(|value| value.trim().parse().ok())
    {
        mfa.period = value;
    }

    project
}

fn hex(color: egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

fn from_hex(value: &str) -> Option<egui::Color32> {
    let digits = value.trim().strip_prefix('#')?;
    if digits.len() != 6 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).ok();
    Some(egui::Color32::from_rgb(
        channel(0..2)?,
        channel(2..4)?,
        channel(4..6)?,
    ))
}

fn data_type_key(data_type: QrDataType) -> &'static str {
    match data_type {
        QrDataType::Url => "url",
        QrDataType::Text => "text",
        QrDataType::Wifi => "wifi",
        QrDataType::VCard => "vcard",
        QrDataType::Calendar => "calendar",
        QrDataType::Messaging => "messaging",
        QrDataType::Mfa => "mfa",
    }
}

fn data_type_from_key(key: &str) -> Option<QrDataType> {
    QrDataType::ALL
        .into_iter()
        .find(|candidate| data_type_key(*candidate) == key.trim())
}

fn wifi_security_key(security: WifiSecurity) -> &'static str {
    match security {
        WifiSecurity::Wpa => "wpa",
        WifiSecurity::Wep => "wep",
        WifiSecurity::None => "none",
    }
}

fn wifi_security_from_key(key: &str) -> Option<WifiSecurity> {
    WifiSecurity::ALL
        .into_iter()
        .find(|candidate| wifi_security_key(*candidate) == key.trim())
}

fn messaging_mode_key(mode: MessagingMode) -> &'static str {
    match mode {
        MessagingMode::Email => "email",
        MessagingMode::Sms => "sms",
        MessagingMode::WhatsApp => "whatsapp",
    }
}

fn messaging_mode_from_key(key: &str) -> Option<MessagingMode> {
    MessagingMode::ALL
        .into_iter()
        .find(|candidate| messaging_mode_key(*candidate) == key.trim())
}

fn algorithm_key(algorithm: Algorithm) -> &'static str {
    match algorithm {
        Algorithm::Sha1 => "sha1",
        Algorithm::Sha256 => "sha256",
        Algorithm::Sha512 => "sha512",
    }
}

fn algorithm_from_key(key: &str) -> Option<Algorithm> {
    Algorithm::ALL
        .into_iter()
        .find(|candidate| algorithm_key(*candidate) == key.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quirpy_payload::messaging::MessagingMode;

    struct TempFile(std::path::PathBuf);

    impl TempFile {
        fn new(name: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "quirpy-{name}-{}-{:?}.qpy",
                std::process::id(),
                std::thread::current().id()
            ));
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn populated() -> ProjectState {
        let mut project = ProjectState {
            name: "Kantoor Wi-Fi".to_owned(),
            data_type: QrDataType::Wifi,
            style: StyleState {
                dark: egui::Color32::from_rgb(0x11, 0x22, 0x33),
                light: egui::Color32::from_rgb(0xEE, 0xDD, 0xCC),
            },
            ..ProjectState::default()
        };

        project.fields.url.url = "https://example.com/a?b=c;d".to_owned();
        project.fields.text.text = "line one\nline two\tünïcode 🎉".to_owned();

        project.fields.wifi.ssid = "Gravia Net".to_owned();
        project.fields.wifi.password = "p@ss=word;semi".to_owned();
        project.fields.wifi.security = WifiSecurity::Wep;
        project.fields.wifi.hidden = true;

        project.fields.vcard.first_name = "Jan".to_owned();
        project.fields.vcard.last_name = "de Vries".to_owned();
        project.fields.vcard.org = "Quirpy B.V.".to_owned();
        project.fields.vcard.title = "Engineer".to_owned();
        project.fields.vcard.phone = "+31 6 12345678".to_owned();
        project.fields.vcard.email = "jan@example.com".to_owned();
        project.fields.vcard.url = "https://example.com".to_owned();
        project.fields.vcard.address = "Straat 1\n1234 AB Rotterdam".to_owned();

        project.fields.calendar.title = "Release".to_owned();
        project.fields.calendar.location = "Kantoor".to_owned();
        project.fields.calendar.description = "Ship it".to_owned();
        project.fields.calendar.start = jiff::civil::date(2026, 8, 20).at(9, 30, 0, 0);
        project.fields.calendar.end = jiff::civil::date(2026, 8, 21).at(17, 45, 0, 0);
        project.fields.calendar.all_day = true;

        project.fields.messaging.mode = MessagingMode::WhatsApp;
        project.fields.messaging.email.to = "to@example.com".to_owned();
        project.fields.messaging.email.subject = "Hoi".to_owned();
        project.fields.messaging.email.body = "Regel 1\nRegel 2".to_owned();
        project.fields.messaging.sms.number = "+31612345678".to_owned();
        project.fields.messaging.sms.message = "SMS body".to_owned();
        project.fields.messaging.whatsapp.number = "+31687654321".to_owned();
        project.fields.messaging.whatsapp.text = "WhatsApp body".to_owned();

        project.fields.mfa.issuer = "Quirpy".to_owned();
        project.fields.mfa.account = "jan@example.com".to_owned();
        project.fields.mfa.secret = "JBSWY3DPEHPK3PXP".to_owned();
        project.fields.mfa.algorithm = Algorithm::Sha512;
        project.fields.mfa.digits = 8;
        project.fields.mfa.period = 60;

        project
    }

    #[test]
    fn round_trips_a_fully_populated_project() {
        let file = TempFile::new("roundtrip");
        let project = populated();
        save(&project, file.path()).expect("save failed");
        assert_eq!(load(file.path()).expect("load failed"), project);
    }

    #[test]
    fn meta_is_plaintext_and_secrets_are_not() {
        let file = TempFile::new("plaintext");
        let project = populated();
        save(&project, file.path()).expect("save failed");

        let contents = std::fs::read_to_string(file.path()).expect("read failed");
        assert!(contents.contains("[meta]"));
        assert!(contents.contains(&format!("app_version={}", env!("CARGO_PKG_VERSION"))));
        assert!(contents.contains("schema_version=1"));
        assert!(contents.contains("checksum="));
        assert!(contents.contains("[fields.wifi]"));
        assert!(contents.contains("password="));
        assert!(!contents.contains("p@ss=word;semi"));
        assert!(!contents.contains("Gravia Net"));
        assert!(!contents.contains("JBSWY3DPEHPK3PXP"));
    }

    #[test]
    fn a_tampered_body_fails_the_checksum() {
        let file = TempFile::new("tampered");
        save(&populated(), file.path()).expect("save failed");

        let contents = std::fs::read_to_string(file.path()).expect("read failed");
        let ssid = ini::Ini::load_from_file(file.path())
            .expect("parse failed")
            .get_from(Some("fields.wifi"), "ssid")
            .expect("no ssid")
            .to_owned();
        let flipped = obfuscate("Andere SSID");
        assert_ne!(ssid, flipped);
        std::fs::write(file.path(), contents.replace(&ssid, &flipped)).expect("write failed");

        assert!(matches!(
            load(file.path()),
            Err(ProjectFileError::ChecksumMismatch)
        ));
        assert_eq!(
            load_ignoring_checksum(file.path())
                .expect("open anyway failed")
                .fields
                .wifi
                .ssid,
            "Andere SSID"
        );
    }

    #[test]
    fn a_newer_schema_is_rejected() {
        let file = TempFile::new("schema");
        save(&populated(), file.path()).expect("save failed");

        let contents = std::fs::read_to_string(file.path()).expect("read failed");
        std::fs::write(
            file.path(),
            contents.replace("schema_version=1", "schema_version=99"),
        )
        .expect("write failed");

        assert!(matches!(
            load(file.path()),
            Err(ProjectFileError::UnsupportedSchema(99))
        ));
    }

    #[test]
    fn awkward_values_survive_the_round_trip() {
        let file = TempFile::new("awkward");
        let mut project = ProjectState {
            name: "= ; [not a section] # \n\t ünïcodé 🔑".to_owned(),
            ..ProjectState::default()
        };
        project.fields.text.text = "a=b\nc;d\r\n[e]\\f".to_owned();
        save(&project, file.path()).expect("save failed");
        assert_eq!(load(file.path()).expect("load failed"), project);
    }

    #[test]
    fn a_default_project_round_trips() {
        let file = TempFile::new("default");
        let project = ProjectState::default();
        save(&project, file.path()).expect("save failed");
        assert_eq!(load(file.path()).expect("load failed"), project);
    }

    #[test]
    fn fields_of_inactive_data_types_are_preserved() {
        let file = TempFile::new("inactive");
        let mut project = populated();
        project.data_type = QrDataType::Url;
        save(&project, file.path()).expect("save failed");

        let loaded = load(file.path()).expect("load failed");
        assert_eq!(loaded.fields.wifi, project.fields.wifi);
        assert_eq!(loaded.fields.mfa, project.fields.mfa);
        assert_eq!(loaded.fields.vcard, project.fields.vcard);
    }
}
