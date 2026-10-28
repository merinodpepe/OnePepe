// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: «Background and lines» (View tab): type of lines, spacing (px or mm), width, color and margin line,
// for the open page or for all the pages of its section. The type, spacing and colors are the background of the
// .rnote format, as in Rnote; the width and the margin are not in the format and go in the hidden file of the page.

// Imports
use super::Engine;
use crate::WidgetFlags;
use crate::document::background::{Background, MarginLine, PatternStyle};
use crate::engine::EngineSnapshot;
use crate::fileformats::FileFormatSaver;
use crate::fileformats::rnoteformat::RnoteFile;
use p2d::math::Vector2;
use rnote_compose::Color;
use serde::{Deserialize, Serialize};

/// The settings of «Background and lines».
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinesSettings {
    pub pattern: PatternStyle,
    /// Distance between the lines, in px (96 per inch).
    pub spacing: f64,
    pub line_width: f64,
    pub color: Color,
    pub background_color: Color,
    /// Distance of the margin line from the left edge of the page, if any.
    pub margin: Option<f64>,
}

impl Default for LinesSettings {
    fn default() -> Self {
        let background = Background::default();
        Self {
            pattern: background.pattern,
            spacing: background.pattern_size[1],
            line_width: background.line_width(),
            color: background.pattern_color,
            background_color: background.color,
            margin: None,
        }
    }
}

impl LinesSettings {
    pub const SPACING_MIN: f64 = 4.0;
    pub const SPACING_MAX: f64 = 400.0;
    pub const WIDTH_MIN: f64 = 0.1;
    pub const WIDTH_MAX: f64 = 8.0;

    pub fn mm_to_px(mm: f64) -> f64 {
        mm / 25.4 * 96.0
    }

    pub fn px_to_mm(px: f64) -> f64 {
        px / 96.0 * 25.4
    }

    /// The named presets: (name, settings).
    pub fn presets() -> Vec<(&'static str, LinesSettings)> {
        let lines = |pattern, mm: f64, margin_mm: Option<f64>| LinesSettings {
            pattern,
            spacing: Self::mm_to_px(mm),
            margin: margin_mm.map(Self::mm_to_px),
            ..Default::default()
        };
        vec![
            ("blank", lines(PatternStyle::None, 8.0, None)),
            ("ruled-7", lines(PatternStyle::Lines, 7.1, Some(25.0))),
            ("ruled-8", lines(PatternStyle::Lines, 8.7, Some(25.0))),
            ("grid-5", lines(PatternStyle::Grid, 5.0, None)),
            ("dots-5", lines(PatternStyle::Dots, 5.0, None)),
        ]
    }

    pub fn from_background(background: &Background) -> Self {
        Self {
            pattern: background.pattern,
            spacing: background.pattern_size[1],
            line_width: background.line_width(),
            color: background.pattern_color,
            background_color: background.color,
            margin: background.onepepe_margin.map(|m| m.x),
        }
    }

    /// Writes the settings into a background, for pages `page_width` wide.
    pub fn apply_to(&self, background: &mut Background, page_width: f64) {
        let spacing = self.spacing.clamp(Self::SPACING_MIN, Self::SPACING_MAX);
        background.pattern = self.pattern;
        background.pattern_size = Vector2::splat(spacing);
        background.pattern_color = self.color;
        background.color = self.background_color;
        let width = self.line_width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
        background.onepepe_line_width =
            ((width - Background::default().line_width()).abs() > 1e-6).then_some(width);
        background.onepepe_margin = self.margin.map(|x| MarginLine {
            x,
            color: background
                .onepepe_margin
                .map(|m| m.color)
                .unwrap_or(MarginLine::COLOR_DEFAULT),
            page_width,
        });
    }
}

impl Engine {
    pub fn lines_settings(&self) -> LinesSettings {
        LinesSettings::from_background(&self.document.config.background)
    }

    /// Applies «Background and lines» to the open page.
    pub fn apply_lines_settings(&mut self, settings: &LinesSettings) -> WidgetFlags {
        let page_width = self.document.config.format.width();
        settings.apply_to(&mut self.document.config.background, page_width);
        let mut widget_flags = self.background_rendering_regenerate();
        widget_flags.store_modified = true;
        widget_flags.redraw = true;
        widget_flags
    }
}

/// Applies «Background and lines» to a page that is not open: the .rnote bytes with the new background (the
/// strokes stay as they are), and the content of its hidden file with the width and the margin.
pub async fn rewrite_page_lines(
    bytes: Vec<u8>,
    file_name: String,
    sidecar: Option<String>,
    settings: LinesSettings,
) -> anyhow::Result<(Vec<u8>, String)> {
    let mut snapshot = EngineSnapshot::load_from_rnote_bytes(bytes).await?;
    let page_width = snapshot.document.config.format.width();
    let mut background = snapshot.document.config.background;
    settings.apply_to(&mut background, page_width);
    snapshot.document.config.background = background;
    let rnote_file = RnoteFile {
        engine_snapshot: ijson::to_value(&snapshot)?,
    };
    let bytes = rnote_file.save_as_bytes(&file_name)?;

    let mut json = sidecar
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({"version": 2, "symbols": []}));
    json["line_width"] = serde_json::to_value(background.onepepe_line_width)?;
    json["margin"] = serde_json::to_value(background.onepepe_margin)?;
    Ok((bytes, json.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip_through_the_background() {
        let settings = LinesSettings {
            pattern: PatternStyle::Lines,
            spacing: LinesSettings::mm_to_px(7.1),
            line_width: 1.2,
            margin: Some(LinesSettings::mm_to_px(25.0)),
            ..Default::default()
        };
        let mut background = Background::default();
        settings.apply_to(&mut background, 794.0);
        assert_eq!(background.onepepe_line_width, Some(1.2));
        assert_eq!(background.onepepe_margin.unwrap().page_width, 794.0);
        let back = LinesSettings::from_background(&background);
        assert!((back.spacing - settings.spacing).abs() < 1e-9);
        assert_eq!(back.margin, settings.margin);
        // the default width is not kept
        LinesSettings::default().apply_to(&mut background, 794.0);
        assert_eq!(background.onepepe_line_width, None);
        assert!((LinesSettings::px_to_mm(LinesSettings::mm_to_px(5.0)) - 5.0).abs() < 1e-9);
        assert_eq!(LinesSettings::presets().len(), 5);
    }

    #[test]
    fn engine_applies_and_keeps_in_the_hidden_file() {
        let mut engine = Engine::default();
        let settings = LinesSettings {
            pattern: PatternStyle::Grid,
            margin: Some(60.0),
            ..Default::default()
        };
        let _ = engine.apply_lines_settings(&settings);
        assert_eq!(engine.lines_settings().pattern, PatternStyle::Grid);
        let json = engine.math_sidecar_json().unwrap();
        assert!(json.contains("\"margin\""));
    }

    #[test]
    fn closed_page_is_rewritten() {
        let engine = Engine::default();
        let bytes = futures::executor::block_on(engine.save_as_rnote_bytes("p.rnote".into()))
            .unwrap()
            .unwrap();
        let settings = LinesSettings {
            pattern: PatternStyle::Dots,
            line_width: 2.0,
            ..Default::default()
        };
        let (new_bytes, sidecar) = futures::executor::block_on(rewrite_page_lines(
            bytes,
            "p.rnote".into(),
            None,
            settings,
        ))
        .unwrap();
        let snapshot =
            futures::executor::block_on(EngineSnapshot::load_from_rnote_bytes(new_bytes)).unwrap();
        assert_eq!(
            snapshot.document.config.background.pattern,
            PatternStyle::Dots
        );
        let mut other = Engine::default();
        let _ = other.load_snapshot(snapshot);
        let _ = other.math_load_sidecar(&sidecar);
        assert_eq!(
            other.document.config.background.onepepe_line_width,
            Some(2.0)
        );
    }
}
