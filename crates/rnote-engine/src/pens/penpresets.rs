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
            pen("Negro", BrushStyle::Solid, Color::BLACK, 2.0),
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
                "Resaltador amarillo",
                BrushStyle::Marker,
                Color::new(1.0, 0.92, 0.0, 1.0),
                16.0,
            ),
        ])
    }
}

impl PenPresets {
    /// The slots for favorite pens (design guide: 3-5, chosen with Ctrl+1…Ctrl+5).
    pub const MAX: usize = 5;

    /// Reads the list from JSON. Falls back to the default pens when the JSON is unreadable. Lists saved before the
    /// slots (up to 24 pens) are read whole; [Self::fit_slots] keeps the first ones.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str::<Self>(json).unwrap_or_default()
    }

    /// Keeps only the pens that fit in the slots. Returns whether some were left out.
    pub fn fit_slots(&mut self) -> bool {
        let over = self.0.len() > Self::MAX;
        self.0.truncate(Self::MAX);
        over
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Saves a favorite pen in the first free slot. Returns false when all the slots are taken.
    pub fn add(&mut self, preset: PenPreset) -> bool {
        if self.0.len() >= Self::MAX {
            return false;
        }
        self.0.push(preset);
        true
    }

    /// Saves a favorite pen in a slot, replacing the pen there (or in the first free slot).
    pub fn replace(&mut self, index: usize, preset: PenPreset) {
        match self.0.get_mut(index) {
            Some(slot) => *slot = preset,
            None => {
                self.add(preset);
            }
        }
    }

    /// Moves the pen of slot `from` to slot `to` (the others shift).
    pub fn move_to(&mut self, from: usize, to: usize) {
        if from < self.0.len() && to < self.0.len() && from != to {
            let preset = self.0.remove(from);
            self.0.insert(to, preset);
        }
    }

    /// The slot of the pen that is the same as `current` (type of stroke, color and width), if any.
    pub fn position_of(&self, current: &PenPreset) -> Option<usize> {
        // the color goes through the color picker of the UI (single precision): compare to 1/255
        let same_color = |a: &Color, b: &Color| {
            [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)]
                .iter()
                .all(|(x, y)| (x - y).abs() < 1.0 / 255.0)
        };
        self.0.iter().position(|p| {
            p.style == current.style
                && same_color(&p.color, &current.color)
                && (p.width - current.width).abs() < 0.05
        })
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
        assert!(!presets.add(PenPreset::default()));
        assert_eq!(presets.0.len(), PenPresets::MAX);
        presets.remove(0);
        presets.remove(999);
        assert_eq!(presets.0.len(), PenPresets::MAX - 1);
        assert!(presets.add(PenPreset::default()));
        // old lists of up to 24 pens keep their first pens
        let mut old = PenPresets(vec![PenPreset::default(); 8]);
        assert!(old.fit_slots());
        assert_eq!(old.0.len(), PenPresets::MAX);
    }

    #[test]
    fn reorder_replace_and_find() {
        let mut presets = PenPresets::default();
        let blue = presets.0[1].clone();
        presets.move_to(1, 3);
        assert_eq!(presets.position_of(&blue), Some(3));
        presets.replace(0, blue.clone());
        assert_eq!(presets.position_of(&blue), Some(0));
        let mut close = blue.clone();
        close.color.r += 0.001;
        assert_eq!(presets.position_of(&close), Some(0));
        presets.move_to(0, 99);
        assert_eq!(presets.0.len(), PenPresets::MAX);
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
