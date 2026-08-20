use crate::quirpy_front::style::{self, ContrastWarning, StyleState};
use crate::quirpy_payload::{
    self, DataFields, calendar::CalendarFields, messaging::MessagingFields,
    messaging::MessagingMode, mfa::Algorithm, mfa::MfaFields, simple::TextFields,
    simple::UrlFields, vcard::VCardFields, wifi::WifiFields, wifi::WifiSecurity,
};

pub use crate::quirpy_payload::QrDataType;

const PAYLOAD_PREVIEW_CHARS: usize = 180;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProjectState {
    pub name: String,
    pub data_type: QrDataType,
    pub fields: DataFields,
    pub style: StyleState,
}

pub fn ui(ui: &mut egui::Ui, project: &mut ProjectState) {
    ui.heading("Data");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("Project name");
        ui.add(
            egui::TextEdit::singleline(&mut project.name)
                .hint_text("Untitled")
                .desired_width(f32::INFINITY),
        );

        ui.add_space(8.0);

        egui::ComboBox::from_label("Data type")
            .selected_text(project.data_type.to_string())
            .show_ui(ui, |ui| {
                for data_type in QrDataType::ALL {
                    if ui
                        .selectable_value(&mut project.data_type, data_type, data_type.to_string())
                        .changed()
                    {
                        tracing::debug!(?data_type, "data type changed");
                    }
                }
            });

        ui.add_space(8.0);

        let changed = match project.data_type {
            QrDataType::Url => url_fields(ui, &mut project.fields.url),
            QrDataType::Text => text_fields(ui, &mut project.fields.text),
            QrDataType::Wifi => wifi_fields(ui, &mut project.fields.wifi),
            QrDataType::VCard => vcard_fields(ui, &mut project.fields.vcard),
            QrDataType::Calendar => calendar_fields(ui, &mut project.fields.calendar),
            QrDataType::Messaging => messaging_fields(ui, &mut project.fields.messaging),
            QrDataType::Mfa => mfa_fields(ui, &mut project.fields.mfa),
        };
        if changed {
            tracing::debug!(data_type = ?project.data_type, "field changed");
        }

        ui.add_space(12.0);
        style_section(ui, &mut project.style);

        ui.add_space(12.0);
        payload_line(ui, project);

        ui.add_space(12.0);
        ui.add_enabled(false, egui::Button::new("Save Project"))
            .on_disabled_hover_text("Coming soon");
    });
}

fn url_fields(ui: &mut egui::Ui, fields: &mut UrlFields) -> bool {
    singleline(ui, "URL", &mut fields.url)
}

fn text_fields(ui: &mut egui::Ui, fields: &mut TextFields) -> bool {
    multiline(ui, "Text", &mut fields.text, 4)
}

fn wifi_fields(ui: &mut egui::Ui, fields: &mut WifiFields) -> bool {
    let mut changed = singleline(ui, "SSID", &mut fields.ssid);

    let mut security = fields.security;
    egui::ComboBox::from_label("Security")
        .selected_text(security.to_string())
        .show_ui(ui, |ui| {
            for option in WifiSecurity::ALL {
                ui.selectable_value(&mut security, option, option.to_string());
            }
        });
    if security != fields.security {
        fields.security = security;
        changed = true;
    }

    if fields.security != WifiSecurity::None {
        changed |= secret_singleline(ui, "Password", "wifi_password", &mut fields.password);
    }
    changed |= ui.checkbox(&mut fields.hidden, "Hidden network").changed();
    changed
}

fn vcard_fields(ui: &mut egui::Ui, fields: &mut VCardFields) -> bool {
    let mut changed = singleline(ui, "First name", &mut fields.first_name);
    changed |= singleline(ui, "Last name", &mut fields.last_name);
    changed |= singleline(ui, "Organisation", &mut fields.org);
    changed |= singleline(ui, "Title", &mut fields.title);
    changed |= singleline(ui, "Phone", &mut fields.phone);
    changed |= singleline(ui, "Email", &mut fields.email);
    changed |= singleline(ui, "Website", &mut fields.url);
    changed |= multiline(ui, "Address", &mut fields.address, 2);
    changed
}

fn calendar_fields(ui: &mut egui::Ui, fields: &mut CalendarFields) -> bool {
    let mut changed = singleline(ui, "Title", &mut fields.title);
    changed |= singleline(ui, "Location", &mut fields.location);
    changed |= multiline(ui, "Description", &mut fields.description, 3);
    changed |= ui.checkbox(&mut fields.all_day, "All day").changed();

    ui.add_space(4.0);
    changed |= date_edit(ui, "Start", "calendar_start", &mut fields.start);
    if !fields.all_day {
        changed |= time_edit(ui, "Start time", &mut fields.start);
    }
    changed |= date_edit(ui, "End", "calendar_end", &mut fields.end);
    if !fields.all_day {
        changed |= time_edit(ui, "End time", &mut fields.end);
    }
    changed
}

fn messaging_fields(ui: &mut egui::Ui, fields: &mut MessagingFields) -> bool {
    let mut changed = false;

    let mut mode = fields.mode;
    egui::ComboBox::from_label("Channel")
        .selected_text(mode.to_string())
        .show_ui(ui, |ui| {
            for option in MessagingMode::ALL {
                ui.selectable_value(&mut mode, option, option.to_string());
            }
        });
    if mode != fields.mode {
        fields.mode = mode;
        changed = true;
    }

    ui.add_space(4.0);

    match fields.mode {
        MessagingMode::Email => {
            changed |= singleline(ui, "To", &mut fields.email.to);
            changed |= singleline(ui, "Subject", &mut fields.email.subject);
            changed |= multiline(ui, "Body", &mut fields.email.body, 4);
        }
        MessagingMode::Sms => {
            changed |= singleline(ui, "Phone number", &mut fields.sms.number);
            changed |= multiline(ui, "Message", &mut fields.sms.message, 3);
        }
        MessagingMode::WhatsApp => {
            changed |= singleline(ui, "Phone number", &mut fields.whatsapp.number);
            changed |= multiline(ui, "Message", &mut fields.whatsapp.text, 3);
        }
    }
    changed
}

fn mfa_fields(ui: &mut egui::Ui, fields: &mut MfaFields) -> bool {
    let mut changed = singleline(ui, "Issuer", &mut fields.issuer);
    changed |= singleline(ui, "Account", &mut fields.account);
    changed |= secret_singleline(ui, "Secret", "mfa_secret", &mut fields.secret);

    let advanced = egui::CollapsingHeader::new("Advanced")
        .default_open(false)
        .show(ui, |ui| {
            let mut algorithm = fields.algorithm;
            egui::ComboBox::from_label("Algorithm")
                .selected_text(algorithm.to_string())
                .show_ui(ui, |ui| {
                    for option in Algorithm::ALL {
                        ui.selectable_value(&mut algorithm, option, option.to_string());
                    }
                });
            let mut inner = algorithm != fields.algorithm;
            fields.algorithm = algorithm;

            inner |= ui
                .horizontal(|ui| {
                    ui.label("Digits");
                    ui.add(egui::DragValue::new(&mut fields.digits).range(6..=10))
                        .changed()
                })
                .inner;
            inner |= ui
                .horizontal(|ui| {
                    ui.label("Period (s)");
                    ui.add(egui::DragValue::new(&mut fields.period).range(15..=300))
                        .changed()
                })
                .inner;
            inner
        })
        .body_returned
        .unwrap_or(false);

    changed || advanced
}

fn style_section(ui: &mut egui::Ui, style: &mut StyleState) {
    egui::CollapsingHeader::new("Style")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.color_edit_button_srgba(&mut style.dark).changed() {
                    tracing::debug!(?style.dark, "dark module colour changed");
                }
                ui.label("Dark modules");
            });
            ui.horizontal(|ui| {
                if ui.color_edit_button_srgba(&mut style.light).changed() {
                    tracing::debug!(?style.light, "light module colour changed");
                }
                ui.label("Light modules");
            });

            match style::check(style) {
                Some(ContrastWarning::Inverted) => {
                    ui.label(
                        egui::RichText::new(
                            "Light modules are darker than dark modules — many scanners reject \
                             inverted codes.",
                        )
                        .color(ui.visuals().error_fg_color),
                    );
                }
                Some(ContrastWarning::TooLow(ratio)) => {
                    ui.label(
                        egui::RichText::new(format!(
                            "Low contrast ({ratio:.1}:1) — scanners may struggle to read this code."
                        ))
                        .color(ui.visuals().warn_fg_color),
                    );
                }
                None => {}
            }
        });
}

// TEMPORARY: delete together with quirpy_encoder::placeholder_matrix once the real encoder
// drives the preview (plan/00-decisions.md, 2026-08-20).
fn payload_line(ui: &mut egui::Ui, project: &ProjectState) {
    ui.separator();
    ui.label("Payload");

    match quirpy_payload::build(project.data_type, &project.fields) {
        Ok(payload) => {
            let shown = if payload.chars().count() > PAYLOAD_PREVIEW_CHARS {
                let head: String = payload.chars().take(PAYLOAD_PREVIEW_CHARS).collect();
                format!("{head}…")
            } else {
                payload.clone()
            };
            ui.add(
                egui::Label::new(egui::RichText::new(shown).monospace())
                    .wrap()
                    .selectable(true),
            )
            .on_hover_text(payload);
        }
        Err(error) => {
            ui.label(egui::RichText::new(error.to_string()).color(ui.visuals().error_fg_color));
        }
    }
}

fn singleline(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY))
        .changed()
}

fn multiline(ui: &mut egui::Ui, label: &str, value: &mut String, rows: usize) -> bool {
    ui.label(label);
    ui.add(
        egui::TextEdit::multiline(value)
            .desired_rows(rows)
            .desired_width(f32::INFINITY),
    )
    .changed()
}

fn secret_singleline(ui: &mut egui::Ui, label: &str, id_salt: &str, value: &mut String) -> bool {
    let id = ui.make_persistent_id(id_salt);
    let mut visible = ui.data_mut(|data| data.get_temp::<bool>(id).unwrap_or(false));

    ui.label(label);
    let changed = ui
        .horizontal(|ui| {
            let response = ui.add(egui::TextEdit::singleline(value).password(!visible));
            if ui
                .button("👁")
                .on_hover_text(if visible { "Hide" } else { "Reveal" })
                .clicked()
            {
                visible = !visible;
            }
            response.changed()
        })
        .inner;

    ui.data_mut(|data| data.insert_temp(id, visible));
    changed
}

fn date_edit(
    ui: &mut egui::Ui,
    label: &str,
    id_salt: &str,
    value: &mut jiff::civil::DateTime,
) -> bool {
    let mut date = value.date();
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui_extras::DatePickerButton::new(&mut date).id_salt(id_salt));
    });

    if date == value.date() {
        false
    } else {
        *value = date.to_datetime(value.time());
        true
    }
}

fn time_edit(ui: &mut egui::Ui, label: &str, value: &mut jiff::civil::DateTime) -> bool {
    let mut hour = value.hour();
    let mut minute = value.minute();

    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(&mut hour)
                .range(0..=23)
                .custom_formatter(|n, _| format!("{:02}", n as i64)),
        );
        ui.label(":");
        ui.add(
            egui::DragValue::new(&mut minute)
                .range(0..=59)
                .custom_formatter(|n, _| format!("{:02}", n as i64)),
        );
    });

    if hour == value.hour() && minute == value.minute() {
        return false;
    }
    match value.with().hour(hour).minute(minute).build() {
        Ok(updated) => {
            *value = updated;
            true
        }
        Err(_) => false,
    }
}
