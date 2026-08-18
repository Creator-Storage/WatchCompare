use serde::{Deserialize, Serialize};

pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    ReferenceLocked,
    Clean,
    Illustrated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RgbaColor(pub u8, pub u8, pub u8, pub u8);

impl RgbaColor {
    pub const BLACK: Self = Self(0, 0, 0, 255);
    pub const WHITE: Self = Self(255, 255, 255, 255);
    pub const RED: Self = Self(221, 20, 40, 255);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub badge_value: String,
    #[serde(default)]
    pub badge_label: String,
    #[serde(default)]
    pub artwork_path: Option<String>,
    pub artwork_color: RgbaColor,
    #[serde(default = "default_title_panel")]
    pub title_panel_color: RgbaColor,
    #[serde(default = "default_description_panel")]
    pub description_panel_color: RgbaColor,
}

fn default_title_panel() -> RgbaColor {
    RgbaColor::WHITE
}

fn default_description_panel() -> RgbaColor {
    RgbaColor(86, 86, 86, 255)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub card_pitch_px: f64,
    pub separator_px: u32,
    pub artwork_bottom_y: u32,
    pub title_top_y: u32,
    pub title_bottom_y: u32,
    pub description_top_y: u32,
    pub description_bottom_y: u32,
    pub title_font_px: f32,
    pub description_font_px: f32,
    pub badge_font_px: f32,
    pub badge_color: RgbaColor,
    pub title_text_color: RgbaColor,
    pub description_text_color: RgbaColor,
    pub background_color: RgbaColor,
    pub scroll_px_per_second: f64,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            card_pitch_px: 477.0,
            separator_px: 6,
            artwork_bottom_y: 871,
            title_top_y: 872,
            title_bottom_y: 964,
            description_top_y: 965,
            description_bottom_y: 1074,
            title_font_px: 40.0,
            description_font_px: 28.0,
            badge_font_px: 34.0,
            badge_color: RgbaColor::RED,
            title_text_color: RgbaColor::BLACK,
            description_text_color: RgbaColor::WHITE,
            background_color: RgbaColor(9, 11, 15, 255),
            scroll_px_per_second: 133.473,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_seconds: f64,
    pub video_bitrate_mbps: u32,
    pub audio_bitrate_kbps: u32,
    #[serde(default)]
    pub soundtrack_path: Option<String>,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 60,
            duration_seconds: 30.0,
            video_bitrate_mbps: 18,
            audio_bitrate_kbps: 192,
            soundtrack_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub format_version: u32,
    pub name: String,
    pub model: ModelKind,
    pub cards: Vec<Card>,
    pub theme: Theme,
    pub export: ExportSettings,
    #[serde(default)]
    pub font_path: Option<String>,
}

impl Project {
    pub fn duration_frames(&self) -> u64 {
        (self.export.duration_seconds.max(0.0) * self.export.fps.max(1) as f64).round() as u64
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != PROJECT_FORMAT_VERSION {
            return Err(format!("unsupported project format {}", self.format_version));
        }
        if self.export.width == 0 || self.export.height == 0 {
            return Err("export dimensions must be non-zero".into());
        }
        if self.export.fps == 0 || self.export.fps > 240 {
            return Err("fps must be between 1 and 240".into());
        }
        if !self.export.duration_seconds.is_finite() || self.export.duration_seconds <= 0.0 {
            return Err("duration must be a positive finite number".into());
        }
        if !self.theme.card_pitch_px.is_finite() || self.theme.card_pitch_px <= 1.0 {
            return Err("card pitch must be positive".into());
        }
        if self.cards.is_empty() {
            return Err("project needs at least one card".into());
        }
        if self.model == ModelKind::ReferenceLocked
            && (self.export.width != 1920 || self.export.height != 1080 || self.export.fps != 60)
        {
            return Err("reference_locked renders at the measured source raster: 1920x1080 at 60 fps".into());
        }
        Ok(())
    }

    pub fn demo() -> Self {
        let colors = [
            RgbaColor(55, 181, 235, 255),
            RgbaColor(239, 172, 61, 255),
            RgbaColor(90, 194, 136, 255),
            RgbaColor(151, 103, 214, 255),
            RgbaColor(225, 94, 112, 255),
        ];
        let labels = [
            ("7M", "YEARS AGO", "Ape Noises And Gestures", "Our chimp ancestors spoke with hoots and gestures"),
            ("400K", "YEARS AGO", "Language Section Of Brain Develops", "The FOXP2 gene gave us the language part of our brain"),
            ("300K", "YEARS AGO", "Voice Box Evolves Fully", "A lower larynx unlocked many more distinct sounds"),
            ("50K", "YEARS AGO", "Symbolic Language", "Rapid vocabulary expansion"),
            ("5K", "YEARS AGO", "Writing Systems", "Speech begins to leave records"),
            ("2026", "TODAY", "Modern Language", "Thousands of living languages"),
        ];
        Self {
            format_version: PROJECT_FORMAT_VERSION,
            name: "Evolution Of Language".into(),
            model: ModelKind::ReferenceLocked,
            cards: labels
                .into_iter()
                .enumerate()
                .map(|(i, (badge_value, badge_label, title, description))| Card {
                    id: format!("card-{}", i + 1),
                    title: title.into(),
                    description: description.into(),
                    badge_value: badge_value.into(),
                    badge_label: badge_label.into(),
                    artwork_path: None,
                    artwork_color: colors[i % colors.len()],
                    title_panel_color: default_title_panel(),
                    description_panel_color: default_description_panel(),
                })
                .collect(),
            theme: Theme::default(),
            export: ExportSettings::default(),
            font_path: None,
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::demo()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_round_trip_is_stable() {
        let project = Project::demo();
        let json = serde_json::to_string_pretty(&project).unwrap();
        let decoded: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, project);
        assert!(decoded.validate().is_ok());
    }

    #[test]
    fn frame_count_uses_export_fps() {
        let mut project = Project::demo();
        project.export.fps = 60;
        project.export.duration_seconds = 2.5;
        assert_eq!(project.duration_frames(), 150);
    }

    #[test]
    fn reference_locked_rejects_non_reference_raster() {
        let mut project = Project::demo();
        project.export.width = 1280;
        assert!(project.validate().unwrap_err().contains("1920x1080"));
    }
}
