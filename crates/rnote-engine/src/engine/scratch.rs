// SPDX-License-Identifier: GPL-3.0-or-later
//! OnePepe: scratch-out. Writing a zigzag over ink erases that ink, without changing the tool (phase 1 of the
//! MyScript-like plan, `PLAN-MYSCRIPT.md`).
//!
//! When a brush stroke ends, if it is a zigzag ([`rnote_compose::scratchout`]) and covers most of some earlier
//! strokes, those strokes go to the trash and the zigzag is removed. The history entry of the zigzag is replaced, so
//! a single undo brings everything back. A zigzag that covers nothing stays as ink (it was maybe a letter).

// Imports
use super::Engine;
use crate::WidgetFlags;
use crate::store::StrokeKey;
use crate::strokes::Stroke;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::scratchout::{ScratchParams, is_scratch_out};
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// When the scratch-out works.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "scratch_out_mode")]
pub enum ScratchOutMode {
    /// Never.
    #[serde(rename = "off")]
    Off,
    /// Only in math mode.
    #[default]
    #[serde(rename = "math_mode")]
    MathMode,
    /// Always, with the brush.
    #[serde(rename = "always")]
    Always,
}

impl Engine {
    /// Smallest zigzag, in surface pixels.
    const SCRATCH_MIN_SIZE: f64 = 24.0;

    pub fn scratch_out_mode(&self) -> ScratchOutMode {
        self.config.read().pens_config.brush_config.scratch_out
    }

    /// Called when a brush stroke has just finished. Returns true when it was a scratch-out (then the zigzag is gone
    /// and the strokes under it are in the trash).
    pub(crate) fn scratch_out_after_stroke(&mut self, widget_flags: &mut WidgetFlags) -> bool {
        let (mode, sensitivity) = {
            let config = self.config.read();
            (
                config.pens_config.brush_config.scratch_out,
                config.pens_config.brush_config.scratch_out_sensitivity,
            )
        };
        let enabled = match mode {
            ScratchOutMode::Off => false,
            ScratchOutMode::MathMode => self.math.mode,
            ScratchOutMode::Always => true,
        };
        if !enabled {
            return false;
        }
        let Some(&key) = self.store.keys_sorted_chrono().last() else {
            return false;
        };
        let Some(Stroke::BrushStroke(zigzag)) = self.store.get_stroke_ref(key) else {
            return false;
        };
        let points = zigzag
            .path
            .clone()
            .into_elements()
            .into_iter()
            .map(|e| e.pos)
            .collect::<Vec<_>>();
        let params = ScratchParams::with_sensitivity(
            sensitivity,
            Self::SCRATCH_MIN_SIZE / self.camera.total_zoom(),
        );
        if !is_scratch_out(&points, params) {
            return false;
        }
        let covered = self.scratch_covered_strokes(key, sensitivity);
        if covered.is_empty() {
            return false;
        }
        self.store.set_trashed_keys(&covered, true);
        self.store.remove_stroke(key);
        // one undo step: the zigzag's history entry becomes the erased state
        *widget_flags |= self.store.update_latest_history_entry(Instant::now());
        widget_flags.store_modified = true;
        widget_flags.redraw = true;
        widget_flags.resize = true;
        true
    }

    /// The strokes that the zigzag covers: most of their box is under the zigzag's box, and the zigzag's ink touches
    /// some of them.
    fn scratch_covered_strokes(&self, zigzag: StrokeKey, sensitivity: f64) -> Vec<StrokeKey> {
        let Some(zz) = self.store.get_stroke_ref(zigzag) else {
            return vec![];
        };
        let zz_bounds = zz.bounds();
        let zz_hitboxes = zz.hitboxes();
        let min_cover = 0.75 - 0.3 * sensitivity.clamp(0.0, 1.0);
        let candidates = self
            .store
            .stroke_keys_as_rendered_intersecting_bounds(zz_bounds)
            .into_iter()
            .filter(|k| *k != zigzag)
            .filter(|k| {
                self.store.get_stroke_ref(*k).is_some_and(|s| {
                    let b = s.bounds();
                    cover(&b, &zz_bounds) >= min_cover
                })
            })
            .collect::<Vec<_>>();
        let touched = candidates.iter().any(|k| {
            self.store.get_stroke_ref(*k).is_some_and(|s| {
                s.hitboxes()
                    .iter()
                    .any(|h| zz_hitboxes.iter().any(|z| h.intersects(z)))
            })
        });
        if touched { candidates } else { vec![] }
    }
}

/// How much of `b` is inside `zone` (0.0 to 1.0). Thin boxes (a "-") count by their length.
fn cover(b: &Aabb, zone: &Aabb) -> f64 {
    let ix = (b.maxs.x.min(zone.maxs.x) - b.mins.x.max(zone.mins.x)).max(0.0);
    let iy = (b.maxs.y.min(zone.maxs.y) - b.mins.y.max(zone.mins.y)).max(0.0);
    let w = (b.maxs.x - b.mins.x).max(1e-9);
    let h = (b.maxs.y - b.mins.y).max(1e-9);
    (ix / w).min(1.0) * (iy / h).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use p2d::math::Vector2;
    use rnote_compose::penevent::PenEvent;
    use rnote_compose::penpath::Element;
    use std::collections::HashSet;
    use std::time::Duration;

    fn draw(engine: &mut Engine, pts: &[(f64, f64)]) -> WidgetFlags {
        let t0 = Instant::now();
        let mut flags = WidgetFlags::default();
        for (i, p) in pts.iter().enumerate() {
            let element = Element::new(Vector2::new(p.0, p.1), 0.5);
            let modifier_keys = HashSet::new();
            let event = if i == pts.len() - 1 {
                PenEvent::Up {
                    element,
                    modifier_keys,
                }
            } else {
                PenEvent::Down {
                    element,
                    modifier_keys,
                }
            };
            flags |= engine
                .handle_pen_event(event, None, t0 + Duration::from_millis(8 * i as u64))
                .1;
        }
        flags
    }

    fn polyline(corners: &[(f64, f64)]) -> Vec<(f64, f64)> {
        let mut out: Vec<(f64, f64)> = corners
            .windows(2)
            .flat_map(|w| {
                (0..10).map(move |i| {
                    let t = i as f64 / 10.0;
                    (
                        w[0].0 + (w[1].0 - w[0].0) * t,
                        w[0].1 + (w[1].1 - w[0].1) * t,
                    )
                })
            })
            .collect();
        out.push(*corners.last().unwrap());
        out
    }

    fn visible(engine: &Engine) -> usize {
        engine.store.stroke_keys_as_rendered().len()
    }

    fn zigzag_over(x0: f64, x1: f64, y0: f64, y1: f64) -> Vec<(f64, f64)> {
        let dy = (y1 - y0) / 5.0;
        polyline(&[
            (x0, y0),
            (x1, y0 + dy),
            (x0, y0 + 2. * dy),
            (x1, y0 + 3. * dy),
            (x0, y0 + 4. * dy),
            (x1, y1),
        ])
    }

    #[test]
    fn a_zigzag_over_ink_erases_it_and_one_undo_brings_it_back() {
        let mut engine = Engine::default();
        engine.config.write().pens_config.brush_config.scratch_out = ScratchOutMode::Always;
        let _ = draw(&mut engine, &polyline(&[(100., 100.), (105., 140.)]));
        let _ = draw(&mut engine, &polyline(&[(120., 100.), (140., 140.)]));
        // a stroke far away is not touched
        let _ = draw(&mut engine, &polyline(&[(400., 100.), (420., 140.)]));
        assert_eq!(visible(&engine), 3);
        let flags = draw(&mut engine, &zigzag_over(95., 145., 95., 145.));
        assert!(flags.store_modified);
        assert_eq!(visible(&engine), 1);
        let _ = engine.undo(Instant::now());
        assert_eq!(visible(&engine), 3);
    }

    #[test]
    fn a_zigzag_over_nothing_stays_as_ink() {
        let mut engine = Engine::default();
        engine.config.write().pens_config.brush_config.scratch_out = ScratchOutMode::Always;
        let _ = draw(&mut engine, &zigzag_over(95., 145., 95., 145.));
        assert_eq!(visible(&engine), 1);
    }

    #[test]
    fn only_in_math_mode_by_default() {
        let mut engine = Engine::default();
        let _ = draw(&mut engine, &polyline(&[(100., 100.), (105., 140.)]));
        let _ = draw(&mut engine, &zigzag_over(95., 145., 95., 145.));
        assert_eq!(visible(&engine), 2);
        let _ = engine.set_math_mode(true);
        let _ = draw(&mut engine, &zigzag_over(95., 145., 95., 145.));
        // the first zigzag was ink under the second one: both go
        assert_eq!(visible(&engine), 0);
    }
}
