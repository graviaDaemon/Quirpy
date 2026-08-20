use crate::quirpy_payload::{PayloadError, escape::escape_wifi};
use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WifiSecurity {
    #[default]
    Wpa,
    Wep,
    None,
}

impl WifiSecurity {
    pub const ALL: [WifiSecurity; 3] = [WifiSecurity::Wpa, WifiSecurity::Wep, WifiSecurity::None];

    fn tag(self) -> &'static str {
        match self {
            WifiSecurity::Wpa => "WPA",
            WifiSecurity::Wep => "WEP",
            WifiSecurity::None => "nopass",
        }
    }
}

impl fmt::Display for WifiSecurity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            WifiSecurity::Wpa => "WPA / WPA2",
            WifiSecurity::Wep => "WEP",
            WifiSecurity::None => "None (open)",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WifiFields {
    pub ssid: String,
    pub password: String,
    pub security: WifiSecurity,
    pub hidden: bool,
}

pub fn build(fields: &WifiFields) -> Result<String, PayloadError> {
    if fields.ssid.is_empty() {
        return Err(PayloadError::MissingField("SSID"));
    }
    if fields.security != WifiSecurity::None && fields.password.is_empty() {
        return Err(PayloadError::MissingField("Password"));
    }

    let mut out = format!(
        "WIFI:T:{};S:{};",
        fields.security.tag(),
        escape_wifi(&fields.ssid)
    );
    if fields.security != WifiSecurity::None {
        out.push_str(&format!("P:{};", escape_wifi(&fields.password)));
    }
    if fields.hidden {
        out.push_str("H:true;");
    }
    out.push(';');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_wpa_network() {
        let fields = WifiFields {
            ssid: "HomeNet".to_owned(),
            password: "hunter2".to_owned(),
            security: WifiSecurity::Wpa,
            hidden: true,
        };
        assert_eq!(
            build(&fields).unwrap(),
            "WIFI:T:WPA;S:HomeNet;P:hunter2;H:true;;"
        );
    }

    #[test]
    fn open_network_omits_password_and_hidden_flag() {
        let fields = WifiFields {
            ssid: "Cafe".to_owned(),
            password: "ignored".to_owned(),
            security: WifiSecurity::None,
            hidden: false,
        };
        assert_eq!(build(&fields).unwrap(), "WIFI:T:nopass;S:Cafe;;");
    }

    #[test]
    fn reserved_characters_are_escaped() {
        let fields = WifiFields {
            ssid: r"Guest;Net\2".to_owned(),
            password: "a,b".to_owned(),
            security: WifiSecurity::Wep,
            hidden: false,
        };
        assert_eq!(
            build(&fields).unwrap(),
            r"WIFI:T:WEP;S:Guest\;Net\\2;P:a\,b;;"
        );
    }

    #[test]
    fn empty_ssid_is_an_error() {
        assert_eq!(
            build(&WifiFields::default()),
            Err(PayloadError::MissingField("SSID"))
        );
    }

    #[test]
    fn secured_network_requires_a_password() {
        let fields = WifiFields {
            ssid: "HomeNet".to_owned(),
            ..Default::default()
        };
        assert_eq!(build(&fields), Err(PayloadError::MissingField("Password")));
    }
}
