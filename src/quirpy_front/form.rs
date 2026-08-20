use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrDataType {
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

    pub fn is_implemented(self) -> bool {
        matches!(self, QrDataType::Url | QrDataType::Text)
    }
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

pub struct FormState {
    pub data_type: QrDataType,
    pub payload: String,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            data_type: QrDataType::Url,
            payload: String::new(),
        }
    }
}

pub fn ui(ui: &mut egui::Ui, form: &mut FormState) {
    ui.heading("Data");
    ui.separator();

    egui::ComboBox::from_label("Data type")
        .selected_text(form.data_type.to_string())
        .show_ui(ui, |ui| {
            for data_type in QrDataType::ALL {
                if ui
                    .selectable_value(&mut form.data_type, data_type, data_type.to_string())
                    .changed()
                {
                    tracing::debug!(?data_type, "data type changed");
                }
            }
        });

    ui.add_space(8.0);

    if form.data_type.is_implemented() {
        ui.label(match form.data_type {
            QrDataType::Url => "URL",
            _ => "Text",
        });
        let response = ui.text_edit_singleline(&mut form.payload);
        if response.changed() {
            tracing::debug!(payload = %form.payload, "payload changed");
        }
    } else {
        ui.add_enabled_ui(false, |ui| {
            ui.label("Coming soon");
        });
    }

    ui.add_space(12.0);
    ui.add_enabled(false, egui::Button::new("Save Project"))
        .on_disabled_hover_text("Coming soon");
}
