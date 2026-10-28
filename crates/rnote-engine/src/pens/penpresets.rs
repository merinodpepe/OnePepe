// OnePepe: favorite pens, like the pen gallery of OneNote: color, width and type of stroke saved together and
// chosen with one click. They are stored in a small JSON file of the app settings, not in the .rnote files.

// Imports
use super::PenStyle;
use super::pensconfig::BrushConfig;
use super::pensconfig::brushconfig::BrushStyle;
use crate::{Engine, WidgetFlags};
use rnote_compose::Color;
use rnote_compose::style::PressureCurve;
use serde::{Deserialize, Serialize};

/// A favorite pen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PenPreset {
    pub name: String,
    pub style: BrushStyle,
    pub color: Color,
    pub width: f64,
    pub pressure_curve: PressureCurve,
}

impl Default for PenPreset {
    fn default() -> Self {
        Self {
            name: String::new(),
            style: BrushStyle::Solid,
            color: Color::BLACK,
            width: 2.0,
            pressure_curve: PressureCurve::Linear,
        }
    }
}

impl PenPreset {
    /// The favorite pen of the current brush settings.
    pub fn from_brush_config(config: &BrushConfig, name: String) -> Self {
        let (color, width, pressure_curve) = match config.style {
            BrushStyle::Marker => (
                config.marker_options.stroke_color,
                config.marker_options.stroke_width,
                config.marker_options.pressure_curve,
            ),
            BrushStyle::Solid => (
                config.solid_options.stroke_color,
                config.solid_options.stroke_width,
                config.solid_options.pressure_curve,
            ),
            BrushStyle::Textured => (
                config.textured_options.stroke_color,
                config.textured_options.stroke_width,
                config.textured_options.pressure_curve,
            ),
        };
        Self {
            name,
            style: config.style,
            color: color.unwrap_or(Color::BLACK),
            width,
            pressure_curve,
        }
    }

    /// Applies the favorite pen to the brush settings.
    pub fn apply_to(&self, config: &mut BrushConfig) {
        let width = self
            .width
            .clamp(BrushConfig::STROKE_WIDTH_MIN, BrushConfig::STROKE_WIDTH_MAX);
        config.style = self.style;
        match self.style {
            BrushStyle::Marker => {
                config.marker_options.stroke_color = Some(self.color);
                config.marker_options.stroke_width = width;
                config.marker_options.pressure_curve = self.pressure_curve;
            }
            BrushStyle::Solid => {
                config.solid_options.stroke_color = Some(self.color);
                config.solid_options.stroke_width = width;
                config.solid_options.pressure_curve = self.pressure_curve;
            }
            BrushStyle::Textured => {
                config.textured_options.stroke_color = Some(self.color);
                config.textured_options.stroke_width = width;
                config.textured_options.pressure_curve = self.pressure_curve;
            }
        }
    }
}

/// The list of favorite pens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenPresets(pub Vec<PenPreset>);

impl Default for PenPresets {
    fn default() -> Self {
        let pen = |name: &str, style, color: Color, width| PenPreset {
            name: name.to_string(),
            style,
            color,
            width,
            pressure_curve: PressureCurve::Linear,
        };
        Self(vec![
            pen("Negro fino", BrushStyle::Solid, Color::BLACK, 1.5),
            pen("Negro", BrushStyle::Solid, Color::BLACK, 2.5),
            pen(
                "Azul",
                BrushStyle::Solid,
                Color::new(0.10, 0.30, 0.80, 1.0),
                2.0,
            ),
            pen(
                "Rojo",
                BrushStyle::Solid,
                Color::new(0.80, 0.10, 0.10, 1.0),
                2.0,
            ),
            pen(
                "Verde",
                BrushStyle::Solid,
                Color::new(0.10, 0.55, 0.20, 1.0),
                2.0,
            ),
            pen(
                "Morado",
                BrushStyle::Solid,
                Color::new(0.48, 0.08, 0.88, 1.0),
                2.0,
            ),
            pen(
                "Resaltador amarillo",
                BrushStyle::Marker,
                Color::new(1.0, 0.92, 0.0, 1.0),
                16.0,
            ),
            pen(
                "Resaltador verde",
                BrushStyle::Marker,
                Color::new(0.45, 0.95, 0.35, 1.0),
                16.0,
            ),
        ])
    }
}

impl PenPresets {
    /// The most favorite pens that are kept.
    pub const MAX: usize = 24;

    /// Reads the list from JSON. Falls back to the default pens when the JSON is unreadable.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str::<Self>(json).unwrap_or_default()
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Adds a favorite pen at the end (the oldest is dropped beyond [Self::MAX]).
    pub fn add(&mut self, preset: PenPreset) {
        self.0.push(preset);
        while self.0.len() > Self::MAX {
            self.0.remove(0);
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.0.len() {
            self.0.remove(index);
        }
    }
}

impl Engine {
    /// Chooses a favorite pen: the brush with its color, width and type of stroke.
    pub fn apply_pen_preset(&mut self, preset: &PenPreset) -> WidgetFlags {
        preset.apply_to(&mut self.config.write().pens_config.brush_config);
        let mut widget_flags = self.change_pen_style(PenStyle::Brush);
        widget_flags.refresh_ui = true;
        widget_flags
    }

    /// The favorite pen of the current brush settings.
    pub fn current_pen_preset(&self, name: String) -> PenPreset {
        PenPreset::from_brush_config(&self.config.read().pens_config.brush_config, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_through_brush_config() {
        let mut config = BrushConfig::default();
        let preset = PenPreset {
            name: "x".into(),
            style: BrushStyle::Marker,
            color: Color::new(1.0, 0.0, 0.0, 1.0),
            width: 12.0,
            pressure_curve: PressureCurve::Const,
        };
        preset.apply_to(&mut config);
        assert_eq!(config.style, BrushStyle::Marker);
        let back = PenPreset::from_brush_config(&config, "x".into());
        assert_eq!(back.width, 12.0);
        assert_eq!(back.color, preset.color);
    }

    #[test]
    fn json_and_limits() {
        let mut presets = PenPresets::default();
        let json = presets.to_json().unwrap();
        assert_eq!(PenPresets::from_json(&json).0.len(), presets.0.len());
        assert_eq!(
            PenPresets::from_json("basura").0.len(),
            PenPresets::default().0.len()
        );
        for _ in 0..40 {
            presets.add(PenPreset::default());
        }
        assert_eq!(presets.0.len(), PenPresets::MAX);
        presets.remove(0);
        presets.remove(999);
        assert_eq!(presets.0.len(), PenPresets::MAX - 1);
    }

    #[test]
    fn engine_applies_preset() {
        let mut engine = Engine::default();
        let presets = PenPresets::default();
        let _ = engine.apply_pen_preset(&presets.0[2]);
        assert_eq!(engine.current_pen_style_w_override(), PenStyle::Brush);
        let current = engine.current_pen_preset("a".into());
        assert_eq!(current.color, presets.0[2].color);
    }
}
