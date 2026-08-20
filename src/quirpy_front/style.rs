const MIN_CONTRAST_RATIO: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleState {
    pub dark: egui::Color32,
    pub light: egui::Color32,
}

impl Default for StyleState {
    fn default() -> Self {
        Self {
            dark: egui::Color32::BLACK,
            light: egui::Color32::WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContrastWarning {
    Inverted,
    TooLow(f32),
}

fn linearise(channel: u8) -> f32 {
    let s = channel as f32 / 255.0;
    if s <= 0.03928 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(color: egui::Color32) -> f32 {
    0.2126 * linearise(color.r()) + 0.7152 * linearise(color.g()) + 0.0722 * linearise(color.b())
}

pub fn check(style: &StyleState) -> Option<ContrastWarning> {
    let dark = relative_luminance(style.dark);
    let light = relative_luminance(style.light);

    if light <= dark {
        return Some(ContrastWarning::Inverted);
    }

    let ratio = (light + 0.05) / (dark + 0.05);
    if ratio < MIN_CONTRAST_RATIO {
        Some(ContrastWarning::TooLow(ratio))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white_is_fine() {
        assert_eq!(check(&StyleState::default()), None);
    }

    #[test]
    fn similar_greys_are_too_low_contrast() {
        let style = StyleState {
            dark: egui::Color32::from_gray(110),
            light: egui::Color32::from_gray(150),
        };
        assert!(matches!(check(&style), Some(ContrastWarning::TooLow(_))));
    }

    #[test]
    fn swapped_colours_are_inverted() {
        let style = StyleState {
            dark: egui::Color32::WHITE,
            light: egui::Color32::BLACK,
        };
        assert_eq!(check(&style), Some(ContrastWarning::Inverted));
    }

    #[test]
    fn identical_colours_are_inverted_rather_than_low_contrast() {
        let style = StyleState {
            dark: egui::Color32::from_gray(128),
            light: egui::Color32::from_gray(128),
        };
        assert_eq!(check(&style), Some(ContrastWarning::Inverted));
    }
}
