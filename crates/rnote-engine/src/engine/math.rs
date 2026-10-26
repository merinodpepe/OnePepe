//! OnePepe: math mode. Handwritten math is recognized by the external helper program `onepepe-math` (see
//! `tools/onepepe-math`); this module keeps the state in the engine, builds the requests and applies the responses.
//!
//! - In math mode, every finished brush stroke is queued. After a short pause the UI takes a request
//!   ([`Engine::math_take_request`]), sends it to the helper and gives the response back
//!   ([`Engine::math_apply_response`]).
//! - Confident symbols are replaced by clean strokes (normal brush strokes, so the `.rnote` format is unchanged).
//!   The original strokes were recorded in the history before, so undo brings them back.
//! - The result of a trailing "=" is shown as an overlay (not ink) when enabled.
//! - Tapping on a recognized symbol in math mode lets the user pick another candidate, for that symbol or its whole
//!   group of look-alike symbols ([`Engine::math_correction_requests`], [`Engine::math_apply_correction`]).

// Imports
use super::Engine;
use crate::WidgetFlags;
use crate::pens::PenStyle;
use crate::store::StrokeKey;
use crate::strokes::{BrushStroke, Stroke};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;
use rnote_compose::penevent::PenEvent;
use rnote_compose::penpath::{Element, PenPath};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// User options of the math recognition (saved in the app settings, not in documents).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default, rename = "math_options")]
pub struct MathOptions {
    /// Replace confidently recognized symbols with clean strokes.
    #[serde(rename = "replace")]
    pub replace: bool,
    /// Minimum confidence (0.0 to 1.0) to replace a symbol.
    #[serde(rename = "threshold")]
    pub threshold: f64,
    /// Show the result of a trailing "=" in gray.
    #[serde(rename = "equals")]
    pub equals: bool,
}

impl Default for MathOptions {
    fn default() -> Self {
        Self {
            replace: true,
            threshold: 0.95,
            equals: false,
        }
    }
}

/// A recognized symbol, remembered during the session for context and corrections.
#[derive(Debug, Clone)]
pub struct MathSymbol {
    pub id: u64,
    /// The strokes that currently draw the symbol (clean strokes, or the original ones when not replaced).
    pub keys: Vec<StrokeKey>,
    pub latex: String,
    pub bounds: Aabb,
    pub candidates: Vec<(String, f64)>,
    /// Look-alike group, per recognition request.
    pub group: (u64, i64),
    /// The handwritten strokes, sent with corrections as examples of the user's handwriting.
    pub original: Vec<Vec<[f64; 2]>>,
}

/// A result shown next to an "=".
#[derive(Debug, Clone)]
pub struct MathOverlay {
    pub anchor: Vector2,
    pub height: f64,
    pub text: String,
    pub latex: String,
}

#[derive(Debug, Clone, Default)]
pub struct MathState {
    pub mode: bool,
    pending: Vec<StrokeKey>,
    in_flight: Vec<(u64, Vec<StrokeKey>)>,
    pub symbols: Vec<MathSymbol>,
    pub overlays: Vec<MathOverlay>,
    next_id: u64,
    tap_start: Option<Vector2>,
    tap_moved: bool,
    /// Where the last tap on a symbol happened, for the UI to show the candidates.
    pub last_tap: Option<Vector2>,
    /// Correction requests sent, by request id: (symbol ids, latex).
    corrections: Vec<(u64, Vec<u64>, String)>,
}

// ---------- hidden file next to the page ----------

/// The hidden file next to a page that remembers its recognized symbols: `.<page file name>.onepepe.json`.
pub fn math_sidecar_path(page: &std::path::Path) -> std::path::PathBuf {
    let name = page
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    page.with_file_name(format!(".{name}.onepepe.json"))
}

#[derive(Debug, Serialize, Deserialize)]
struct SidecarFile {
    version: u32,
    symbols: Vec<SidecarSymbol>,
    #[serde(default)]
    overlays: Vec<SidecarOverlay>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SidecarSymbol {
    id: u64,
    /// The strokes of the symbol in the page, and their bounds when saved, to check they did not change.
    keys: Vec<StrokeKey>,
    keys_bounds: [f64; 4],
    latex: String,
    bounds: [f64; 4],
    candidates: Vec<(String, f64)>,
    group: (u64, i64),
    original: Vec<Vec<[f64; 2]>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SidecarOverlay {
    anchor: [f64; 2],
    height: f64,
    text: String,
    latex: String,
}

fn aabb_to_array(b: Aabb) -> [f64; 4] {
    [b.mins.x, b.mins.y, b.maxs.x, b.maxs.y]
}

fn array_to_aabb(a: [f64; 4]) -> Aabb {
    Aabb::new(Vector2::new(a[0], a[1]), Vector2::new(a[2], a[3]))
}

// ---------- protocol ----------

#[derive(Debug, Serialize)]
struct RecognizeRequest<'a> {
    id: u64,
    cmd: &'a str,
    strokes: Vec<Vec<[f64; 2]>>,
    known: Vec<KnownSymbol>,
    notebook_dir: Option<String>,
    threshold: f64,
    clean: bool,
    equals: bool,
}

#[derive(Debug, Serialize)]
struct KnownSymbol {
    latex: String,
    r#box: [f64; 4],
}

#[derive(Debug, Serialize)]
struct CorrectRequest<'a> {
    id: u64,
    cmd: &'a str,
    strokes: Vec<Vec<[f64; 2]>>,
    latex: String,
    notebook_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    latex: String,
    p: f64,
}

#[derive(Debug, Deserialize)]
struct RecognizedSymbol {
    strokes: Vec<usize>,
    candidates: Vec<Candidate>,
    #[serde(default)]
    group: i64,
    #[serde(default)]
    clean: Option<Vec<Vec<[f64; 2]>>>,
}

#[derive(Debug, Deserialize)]
struct MathResult {
    text: String,
    latex: String,
}

#[derive(Debug, Deserialize)]
struct RecognizeResponse {
    id: u64,
    #[serde(default)]
    symbols: Vec<RecognizedSymbol>,
    #[serde(default)]
    result: Option<MathResult>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CorrectResponse {
    id: u64,
    #[serde(default)]
    clean: Option<Vec<Vec<[f64; 2]>>>,
    #[serde(default)]
    error: Option<String>,
}

impl Engine {
    /// How long a tap may move (in surface pixels) to still count as a tap.
    const MATH_TAP_TOLERANCE: f64 = 6.0;

    pub fn math_mode(&self) -> bool {
        self.math.mode
    }

    pub fn set_math_mode(&mut self, mode: bool) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.math.mode = mode;
        if !mode {
            // ink written in math mode stays as it is and is not analyzed anymore
            self.math.pending.clear();
            self.math.tap_start = None;
        }
        widget_flags.redraw = true;
        widget_flags.refresh_ui = true;
        widget_flags
    }

    /// Where the last tap on a recognized symbol happened.
    pub fn math_last_tap(&self) -> Option<Vector2> {
        self.math.last_tap
    }

    pub fn math_options(&self) -> MathOptions {
        self.config.read().pens_config.brush_config.math
    }

    /// Called after every pen event: queues the strokes finished with the brush in math mode.
    pub(crate) fn math_after_pen_event(&mut self, event: &PenEvent) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if !self.math.mode || !matches!(event, PenEvent::Up { .. }) {
            return widget_flags;
        }
        if let Some(&key) = self.store.keys_sorted_chrono().last()
            && matches!(self.store.get_stroke_ref(key), Some(Stroke::BrushStroke(_)))
            && !self.math.pending.contains(&key)
            && !self.math.symbols.iter().any(|s| s.keys.contains(&key))
        {
            self.math.pending.push(key);
            widget_flags.math_pending = true;
        }
        widget_flags
    }

    /// In math mode, a tap with the brush on a recognized symbol opens its candidates instead of drawing.
    ///
    /// Returns None when the event was used for the tap.
    pub(crate) fn math_filter_pen_event(
        &mut self,
        event: PenEvent,
        widget_flags: &mut WidgetFlags,
    ) -> Option<PenEvent> {
        if !self.math.mode
            || self
                .penholder
                .current_pen_style_w_override(&crate::engine_view!(self))
                != PenStyle::Brush
        {
            return Some(event);
        }
        let tolerance = Self::MATH_TAP_TOLERANCE / self.camera.total_zoom();
        match &event {
            PenEvent::Down { element, .. } => {
                if let Some(start) = self.math.tap_start {
                    if (element.pos - start).length() > tolerance {
                        self.math.tap_moved = true;
                    }
                    return None;
                }
                if self.math_symbol_index_at(element.pos).is_some()
                    && self.penholder.current_pen_progress()
                        != rnote_compose::penevent::PenProgress::InProgress
                {
                    self.math.tap_start = Some(element.pos);
                    self.math.tap_moved = false;
                    return None;
                }
                Some(event)
            }
            PenEvent::Up { element, .. } => {
                if let Some(start) = self.math.tap_start.take() {
                    if !self.math.tap_moved && (element.pos - start).length() <= tolerance {
                        self.math.last_tap = Some(start);
                        widget_flags.math_tap = true;
                    }
                    return None;
                }
                Some(event)
            }
            PenEvent::Cancel => {
                self.math.tap_start = None;
                Some(event)
            }
            _ => Some(event),
        }
    }

    fn math_symbol_index_at(&self, pos: Vector2) -> Option<usize> {
        let loosen = 4.0 / self.camera.total_zoom();
        self.math
            .symbols
            .iter()
            .position(|s| s.bounds.loosened(loosen).contains_local_point(pos))
    }

    /// The symbol at the position: (symbol id, candidates).
    pub fn math_symbol_at(&self, pos: Vector2) -> Option<(u64, Vec<(String, f64)>)> {
        self.math_symbol_index_at(pos).map(|i| {
            (
                self.math.symbols[i].id,
                self.math.symbols[i].candidates.clone(),
            )
        })
    }

    fn brush_points(&self, key: StrokeKey) -> Option<Vec<[f64; 2]>> {
        match self.store.get_stroke_ref(key)? {
            Stroke::BrushStroke(b) => Some(
                b.path
                    .clone()
                    .into_elements()
                    .into_iter()
                    .map(|e| [e.pos.x, e.pos.y])
                    .collect(),
            ),
            _ => None,
        }
    }

    /// Takes the queued strokes and builds a recognition request (one JSON line), or None if there is nothing to do.
    pub fn math_take_request(&mut self, notebook_dir: Option<String>) -> Option<String> {
        let keys = std::mem::take(&mut self.math.pending)
            .into_iter()
            .filter(|k| self.brush_points(*k).is_some())
            .collect::<Vec<StrokeKey>>();
        if keys.is_empty() {
            return None;
        }
        let strokes = keys
            .iter()
            .filter_map(|k| self.brush_points(*k))
            .collect::<Vec<_>>();
        let bounds = self.store.bounds_for_strokes(&keys)?;
        // the already recognized symbols of the same line give context
        let band = bounds.extents().y.max(1.0);
        let known = self
            .math
            .symbols
            .iter()
            .filter(|s| {
                s.bounds.maxs.y > bounds.mins.y - 0.5 * band
                    && s.bounds.mins.y < bounds.maxs.y + 0.5 * band
                    && s.keys
                        .iter()
                        .all(|k| self.store.get_stroke_ref(*k).is_some())
            })
            .map(|s| KnownSymbol {
                latex: s.latex.clone(),
                r#box: [
                    s.bounds.mins.x,
                    s.bounds.mins.y,
                    s.bounds.maxs.x,
                    s.bounds.maxs.y,
                ],
            })
            .collect();
        let options = self.math_options();
        self.math.next_id += 1;
        let id = self.math.next_id;
        self.math.in_flight.push((id, keys));
        let request = RecognizeRequest {
            id,
            cmd: "recognize",
            strokes,
            known,
            notebook_dir,
            threshold: options.threshold,
            clean: options.replace,
            equals: options.equals,
        };
        serde_json::to_string(&request).ok()
    }

    /// Inserts clean strokes with the style of `like`, returns their keys.
    fn math_insert_clean(&mut self, lines: &[Vec<[f64; 2]>], like: StrokeKey) -> Vec<StrokeKey> {
        let (style, pressure) = match self.store.get_stroke_ref(like) {
            Some(Stroke::BrushStroke(b)) => {
                let elements = b.path.clone().into_elements();
                let pressure =
                    elements.iter().map(|e| e.pressure).sum::<f64>() / elements.len().max(1) as f64;
                (b.style.clone(), pressure)
            }
            _ => return vec![],
        };
        let layer = self.store.stroke_layer(like);
        let mut keys = vec![];
        for line in lines {
            let elements = line
                .iter()
                .map(|p| Element::new(Vector2::new(p[0], p[1]), pressure));
            if let Some(path) = PenPath::try_from_elements(elements) {
                let stroke = Stroke::BrushStroke(BrushStroke::from_penpath(path, style.clone()));
                let key = self.store.insert_stroke(stroke, layer);
                self.store.regenerate_rendering_for_stroke(
                    key,
                    self.camera.viewport(),
                    self.camera.image_scale(),
                );
                keys.push(key);
            }
        }
        keys
    }

    /// Applies a recognition response (one JSON line from the helper).
    pub fn math_apply_response(&mut self, line: &str) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        let Ok(resp) = serde_json::from_str::<RecognizeResponse>(line) else {
            tracing::warn!("Invalid response from the math helper: {line}");
            return widget_flags;
        };
        let Some(pos) = self
            .math
            .in_flight
            .iter()
            .position(|(id, _)| *id == resp.id)
        else {
            return widget_flags;
        };
        let (req_id, keys) = self.math.in_flight.remove(pos);
        if let Some(e) = resp.error {
            tracing::warn!("Math helper error: {e}");
            return widget_flags;
        }
        let options = self.math_options();
        let mut modified = false;
        let mut equals_box: Option<Aabb> = None;
        for sym in resp.symbols {
            let sym_keys = sym
                .strokes
                .iter()
                .filter_map(|&i| keys.get(i).copied())
                .filter(|k| self.store.get_stroke_ref(*k).is_some())
                .collect::<Vec<StrokeKey>>();
            // strokes erased in the meantime: forget the symbol
            if sym_keys.len() != sym.strokes.len() || sym_keys.is_empty() {
                continue;
            }
            let Some(bounds) = self.store.bounds_for_strokes(&sym_keys) else {
                continue;
            };
            let original = sym_keys
                .iter()
                .filter_map(|k| self.brush_points(*k))
                .collect::<Vec<_>>();
            let candidates = sym
                .candidates
                .iter()
                .map(|c| (c.latex.clone(), c.p))
                .collect::<Vec<_>>();
            let Some(latex) = candidates.first().map(|c| c.0.clone()) else {
                continue;
            };
            let mut current_keys = sym_keys.clone();
            if options.replace
                && let Some(lines) = sym.clean.as_ref()
            {
                let new_keys = self.math_insert_clean(lines, sym_keys[0]);
                if !new_keys.is_empty() {
                    for k in &sym_keys {
                        self.store.remove_stroke(*k);
                    }
                    current_keys = new_keys;
                    modified = true;
                }
            }
            if latex == "=" {
                equals_box = Some(bounds);
            }
            self.math.next_id += 1;
            self.math.symbols.push(MathSymbol {
                id: self.math.next_id,
                keys: current_keys,
                latex,
                bounds,
                candidates,
                group: (req_id, sym.group),
                original,
            });
        }
        if options.equals
            && let (Some(result), Some(eq)) = (resp.result, equals_box)
        {
            let anchor = Vector2::new(eq.maxs.x + 0.3 * eq.extents().x.max(8.0), eq.center().y);
            // one result per "="
            self.math
                .overlays
                .retain(|o| (o.anchor - anchor).length() > eq.extents().x.max(8.0));
            self.math.overlays.push(MathOverlay {
                anchor,
                height: eq.extents().x.max(eq.extents().y) * 1.2,
                text: result.text,
                latex: result.latex,
            });
            widget_flags.redraw = true;
        }
        if modified {
            widget_flags |= self.record(Instant::now());
            widget_flags.store_modified = true;
            widget_flags.redraw = true;
            widget_flags.resize = true;
        }
        widget_flags
    }

    /// Correction requests (JSON lines) for choosing `latex` for the symbol, or for all symbols of its group.
    pub fn math_correction_requests(
        &mut self,
        symbol_id: u64,
        latex: &str,
        whole_group: bool,
        notebook_dir: Option<String>,
    ) -> Vec<String> {
        let Some(sym) = self.math.symbols.iter().find(|s| s.id == symbol_id) else {
            return vec![];
        };
        let group = sym.group;
        let targets = self
            .math
            .symbols
            .iter()
            .filter(|s| s.id == symbol_id || (whole_group && s.group == group))
            .map(|s| (s.id, s.original.clone()))
            .collect::<Vec<_>>();
        let mut out = vec![];
        for (id, original) in targets {
            self.math.next_id += 1;
            let req_id = self.math.next_id;
            self.math
                .corrections
                .push((req_id, vec![id], latex.to_string()));
            if let Ok(s) = serde_json::to_string(&CorrectRequest {
                id: req_id,
                cmd: "correct",
                strokes: original,
                latex: latex.to_string(),
                notebook_dir: notebook_dir.clone(),
            }) {
                out.push(s);
            }
        }
        out
    }

    /// Applies the response to a correction: the symbol is redrawn with the clean glyph of the chosen label.
    pub fn math_apply_correction(&mut self, line: &str) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        let Ok(resp) = serde_json::from_str::<CorrectResponse>(line) else {
            return widget_flags;
        };
        let Some(pos) = self
            .math
            .corrections
            .iter()
            .position(|(id, _, _)| *id == resp.id)
        else {
            return widget_flags;
        };
        let (_, symbol_ids, latex) = self.math.corrections.remove(pos);
        if resp.error.is_some() {
            return widget_flags;
        }
        for sid in symbol_ids {
            let Some(idx) = self.math.symbols.iter().position(|s| s.id == sid) else {
                continue;
            };
            let old_keys = self.math.symbols[idx].keys.clone();
            let Some(&like) = old_keys
                .iter()
                .find(|k| self.store.get_stroke_ref(**k).is_some())
            else {
                continue;
            };
            if let Some(lines) = resp.clean.as_ref() {
                let new_keys = self.math_insert_clean(lines, like);
                if !new_keys.is_empty() {
                    for k in &old_keys {
                        self.store.remove_stroke(*k);
                    }
                    self.math.symbols[idx].keys = new_keys;
                    widget_flags.store_modified = true;
                }
            }
            let sym = &mut self.math.symbols[idx];
            sym.latex = latex.clone();
            // the chosen label goes first
            sym.candidates.retain(|c| c.0 != latex);
            sym.candidates.insert(0, (latex.clone(), 1.0));
        }
        if widget_flags.store_modified {
            widget_flags |= self.record(Instant::now());
            widget_flags.redraw = true;
        }
        widget_flags
    }

    /// The content of the hidden file next to the page (recognized symbols and results), or None if there is nothing
    /// to remember (then an existing hidden file should be removed).
    pub fn math_sidecar_json(&self) -> Option<String> {
        let symbols = self
            .math
            .symbols
            .iter()
            .filter_map(|s| {
                let keys_bounds = self.store.bounds_for_strokes(&s.keys)?;
                if s.keys
                    .iter()
                    .any(|k| self.store.get_stroke_ref(*k).is_none())
                {
                    return None;
                }
                Some(SidecarSymbol {
                    id: s.id,
                    keys: s.keys.clone(),
                    keys_bounds: aabb_to_array(keys_bounds),
                    latex: s.latex.clone(),
                    bounds: aabb_to_array(s.bounds),
                    candidates: s.candidates.clone(),
                    group: s.group,
                    original: s.original.clone(),
                })
            })
            .collect::<Vec<_>>();
        if symbols.is_empty() && self.math.overlays.is_empty() {
            return None;
        }
        let overlays = self
            .math
            .overlays
            .iter()
            .map(|o| SidecarOverlay {
                anchor: [o.anchor.x, o.anchor.y],
                height: o.height,
                text: o.text.clone(),
                latex: o.latex.clone(),
            })
            .collect();
        serde_json::to_string(&SidecarFile {
            version: 1,
            symbols,
            overlays,
        })
        .ok()
    }

    /// Restores the recognized symbols from the hidden file next to the page. Symbols whose strokes were changed
    /// or deleted outside of OnePepe are skipped. Returns how many symbols were restored.
    pub fn math_load_sidecar(&mut self, json: &str) -> usize {
        let Ok(file) = serde_json::from_str::<SidecarFile>(json) else {
            tracing::warn!("Could not read the hidden math file of the page");
            return 0;
        };
        let tolerance = 0.5;
        let mut restored = 0;
        for s in file.symbols {
            let Some(current) = self.store.bounds_for_strokes(&s.keys) else {
                continue;
            };
            let unchanged = s
                .keys
                .iter()
                .all(|k| self.store.get_stroke_ref(*k).is_some())
                && aabb_to_array(current)
                    .iter()
                    .zip(s.keys_bounds.iter())
                    .all(|(a, b)| (a - b).abs() <= tolerance);
            if !unchanged {
                continue;
            }
            self.math.next_id = self.math.next_id.max(s.id).max(s.group.0);
            self.math.symbols.push(MathSymbol {
                id: s.id,
                keys: s.keys,
                latex: s.latex,
                bounds: array_to_aabb(s.bounds),
                candidates: s.candidates,
                group: s.group,
                original: s.original,
            });
            restored += 1;
        }
        self.math.overlays = file
            .overlays
            .into_iter()
            .map(|o| MathOverlay {
                anchor: Vector2::new(o.anchor[0], o.anchor[1]),
                height: o.height,
                text: o.text,
                latex: o.latex,
            })
            .collect();
        restored
    }

    /// Forgets all recognized symbols and results (e.g. when another document is loaded).
    pub(crate) fn math_reset_document(&mut self) {
        let mode = self.math.mode;
        self.math = MathState {
            mode,
            ..MathState::default()
        };
    }

    /// Removes the result overlays (e.g. when the document changes a lot).
    pub fn math_clear_overlays(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.math.overlays.clear();
        widget_flags.redraw = true;
        widget_flags
    }

    /// Draws the math mode border and the "=" results on top of the document.
    #[cfg(feature = "ui")]
    pub(crate) fn draw_math_to_gtk_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
    ) -> anyhow::Result<()> {
        use crate::ext::GrapheneRectExt;
        use gtk4::{graphene, prelude::*};
        use piet::RenderContext;
        use rnote_compose::ext::DAffine2Ext;

        if !self.math.mode && self.math.overlays.is_empty() {
            return Ok(());
        }
        let size = self.camera.size();
        let surface = Aabb::new(Vector2::ZERO, size);
        snapshot.save();
        let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(surface));
        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);
        if self.math.mode {
            draw_math_mode_border(&mut piet_cx, size)?;
        }
        piet_cx.transform(self.camera.transform().to_kurbo());
        draw_math_overlays(&mut piet_cx, &self.math.overlays)?;
        piet_cx.finish().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        snapshot.restore();
        Ok(())
    }
}

/// A colored border around the whole view, so math mode is not forgotten. In surface coordinates.
pub fn draw_math_mode_border(
    piet_cx: &mut impl piet::RenderContext,
    size: Vector2,
) -> anyhow::Result<()> {
    use rnote_compose::color;
    let w = 6.0;
    let rect = kurbo::Rect::new(w * 0.5, w * 0.5, size.x - w * 0.5, size.y - w * 0.5);
    piet_cx.stroke(rect, &color::GNOME_ORANGES[3].with_a8(220), w);
    Ok(())
}

/// The results of "=", in gray, in document coordinates.
pub fn draw_math_overlays(
    piet_cx: &mut impl piet::RenderContext,
    overlays: &[MathOverlay],
) -> anyhow::Result<()> {
    use piet::{Text, TextLayoutBuilder};
    use rnote_compose::color;
    for o in overlays {
        let font_size = o.height.max(6.0);
        if let Ok(layout) = piet_cx
            .text()
            .new_text_layout(o.text.clone())
            .text_color(color::GNOME_DARKS[0].with_a8(150))
            .font(piet::FontFamily::SANS_SERIF, font_size)
            .build()
        {
            piet_cx.draw_text(
                &layout,
                kurbo::Point::new(o.anchor.x, o.anchor.y - font_size * 0.6),
            );
        }
        if let Ok(layout) = piet_cx
            .text()
            .new_text_layout(format!("LaTeX: {}", o.latex))
            .text_color(color::GNOME_DARKS[0].with_a8(110))
            .font(piet::FontFamily::MONOSPACE, font_size * 0.35)
            .build()
        {
            piet_cx.draw_text(
                &layout,
                kurbo::Point::new(o.anchor.x, o.anchor.y + font_size * 0.6),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn line(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<(f64, f64)> {
        (0..=10)
            .map(|i| {
                let t = i as f64 / 10.0;
                (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t)
            })
            .collect()
    }

    fn n_strokes(engine: &Engine) -> usize {
        engine.store.stroke_keys_as_rendered().len()
    }

    /// Writes "1 =" in math mode and answers the request like the helper would.
    fn write_one_equals(engine: &mut Engine) -> WidgetFlags {
        let _ = engine.set_math_mode(true);
        engine.config.write().pens_config.brush_config.math.equals = true;
        assert!(draw(engine, &line(100.0, 100.0, 103.0, 140.0)).math_pending);
        let _ = draw(engine, &line(120.0, 115.0, 140.0, 115.0));
        let _ = draw(engine, &line(120.0, 125.0, 140.0, 125.0));
        assert_eq!(n_strokes(engine), 3);
        let req = engine.math_take_request(Some("/tmp".into())).unwrap();
        let v: serde_json::Value = serde_json::from_str(&req).unwrap();
        assert_eq!(v["cmd"], "recognize");
        assert_eq!(v["strokes"].as_array().unwrap().len(), 3);
        let id = v["id"].as_u64().unwrap();
        // nothing left to send
        assert!(engine.math_take_request(None).is_none());
        let resp = format!(
            r#"{{"id": {id}, "symbols": [
                {{"strokes": [0], "candidates": [{{"latex": "1", "p": 0.99}}, {{"latex": "l", "p": 0.01}}], "group": 0,
                  "clean": [[[101, 100], [101, 140]], [[96, 108], [101, 100]]]}},
                {{"strokes": [1, 2], "candidates": [{{"latex": "=", "p": 0.6}}], "group": 1, "clean": null}}
              ],
              "latex": "1=", "result": {{"text": "1", "latex": "1"}}}}"#
        );
        engine.math_apply_response(&resp)
    }

    #[test]
    fn replaces_confident_symbols_and_undo_restores() {
        let mut engine = Engine::default();
        assert!(write_one_equals(&mut engine).store_modified);
        // the "1" (1 stroke) became 2 clean strokes, the "=" stays as it was
        assert_eq!(n_strokes(&engine), 4);
        assert_eq!(engine.math.symbols.len(), 2);
        assert_eq!(engine.math.overlays.len(), 1);
        assert_eq!(engine.math.overlays[0].text, "1");
        let _ = engine.undo(Instant::now());
        assert_eq!(n_strokes(&engine), 3);
    }

    #[test]
    fn no_replacement_when_disabled() {
        let mut engine = Engine::default();
        engine.config.write().pens_config.brush_config.math.replace = false;
        assert!(!write_one_equals(&mut engine).store_modified);
        assert_eq!(n_strokes(&engine), 3);
        // still remembered for context and corrections
        assert_eq!(engine.math.symbols.len(), 2);
    }

    #[test]
    fn tap_on_symbol_and_group_correction() {
        let mut engine = Engine::default();
        let _ = write_one_equals(&mut engine);
        let before = n_strokes(&engine);
        // a tap on the "1" does not draw, it asks for the candidates
        let flags = draw(&mut engine, &[(101.0, 120.0), (101.5, 120.5)]);
        assert!(flags.math_tap);
        assert_eq!(n_strokes(&engine), before);
        let (id, cands) = engine
            .math_symbol_at(engine.math.last_tap.unwrap())
            .unwrap();
        assert_eq!(cands[0].0, "1");
        let reqs = engine.math_correction_requests(id, "l", true, None);
        assert_eq!(reqs.len(), 1);
        let v: serde_json::Value = serde_json::from_str(&reqs[0]).unwrap();
        assert_eq!(v["cmd"], "correct");
        assert_eq!(v["latex"], "l");
        let resp = format!(
            r#"{{"id": {}, "ok": true, "clean": [[[101, 100], [101, 140]]]}}"#,
            v["id"]
        );
        let flags = engine.math_apply_correction(&resp);
        assert!(flags.store_modified);
        let (_, cands) = engine.math_symbol_at(Vector2::new(101.0, 120.0)).unwrap();
        assert_eq!(cands[0].0, "l");
        // the "1" (2 clean strokes) is now an "l" (1 clean stroke)
        assert_eq!(n_strokes(&engine), before - 1);
    }

    #[test]
    fn writing_elsewhere_still_draws_in_math_mode() {
        let mut engine = Engine::default();
        let _ = write_one_equals(&mut engine);
        let before = n_strokes(&engine);
        let _ = draw(&mut engine, &line(300.0, 100.0, 330.0, 140.0));
        assert_eq!(n_strokes(&engine), before + 1);
    }

    #[test]
    fn leaving_math_mode_drops_pending_strokes() {
        let mut engine = Engine::default();
        let _ = engine.set_math_mode(true);
        let _ = draw(&mut engine, &line(0.0, 0.0, 10.0, 40.0));
        let _ = engine.set_math_mode(false);
        assert!(engine.math_take_request(None).is_none());
        // and nothing is queued outside math mode
        assert!(!draw(&mut engine, &line(50.0, 0.0, 60.0, 40.0)).math_pending);
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn border_and_result_are_drawn() {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 300, 120).unwrap();
        {
            let cx = cairo::Context::new(&surface).unwrap();
            cx.set_source_rgb(1.0, 1.0, 1.0);
            cx.paint().unwrap();
            let mut piet_cx = piet_cairo::CairoRenderContext::new(&cx);
            draw_math_mode_border(&mut piet_cx, Vector2::new(300.0, 120.0)).unwrap();
            let overlays = [MathOverlay {
                anchor: Vector2::new(60.0, 60.0),
                height: 30.0,
                text: "x = 3".into(),
                latex: "x = 3".into(),
            }];
            draw_math_overlays(&mut piet_cx, &overlays).unwrap();
            piet::RenderContext::finish(&mut piet_cx).unwrap();
        }
        if let Ok(path) = std::env::var("ONEPEPE_RENDER_MATH") {
            surface
                .write_to_png(&mut std::fs::File::create(path).unwrap())
                .unwrap();
        }
        let data = surface.take_data().unwrap();
        let not_white = data.chunks(4).filter(|px| px[0] < 200).count();
        assert!(not_white > 1000);
    }
}

#[cfg(test)]
mod sidecar_tests {
    use super::*;

    #[test]
    fn sidecar_path_is_hidden_next_to_the_page() {
        assert_eq!(
            math_sidecar_path(std::path::Path::new("/a/Tema 1/Página 1.rnote")),
            std::path::PathBuf::from("/a/Tema 1/.Página 1.rnote.onepepe.json")
        );
    }

    /// Engine with "1 =" recognized and the "1" replaced, as in the math mode tests.
    fn engine_with_symbols() -> Engine {
        let mut engine = Engine::default();
        let _ = engine.set_math_mode(true);
        engine.config.write().pens_config.brush_config.math.equals = true;
        let lines = [
            [(100.0, 100.0), (103.0, 140.0)],
            [(120.0, 115.0), (140.0, 115.0)],
            [(120.0, 125.0), (140.0, 125.0)],
        ];
        for l in lines {
            let path = PenPath::try_from_elements((0..=10).map(|i| {
                let t = i as f64 / 10.0;
                Element::new(
                    Vector2::new(
                        l[0].0 + (l[1].0 - l[0].0) * t,
                        l[0].1 + (l[1].1 - l[0].1) * t,
                    ),
                    0.5,
                )
            }))
            .unwrap();
            let key = engine.store.insert_stroke(
                Stroke::BrushStroke(BrushStroke::from_penpath(
                    path,
                    rnote_compose::Style::default(),
                )),
                None,
            );
            engine.math.pending.push(key);
        }
        let req = engine.math_take_request(None).unwrap();
        let id = serde_json::from_str::<serde_json::Value>(&req).unwrap()["id"]
            .as_u64()
            .unwrap();
        let resp = format!(
            r#"{{"id": {id}, "symbols": [
                {{"strokes": [0], "candidates": [{{"latex": "1", "p": 0.99}}, {{"latex": "l", "p": 0.01}}], "group": 0,
                  "clean": [[[101, 100], [101, 140]]]}},
                {{"strokes": [1, 2], "candidates": [{{"latex": "=", "p": 0.6}}], "group": 1, "clean": null}}
              ], "result": {{"text": "1", "latex": "1"}}}}"#
        );
        let _ = engine.math_apply_response(&resp);
        engine
    }

    #[test]
    fn symbols_survive_save_and_reopen() {
        let engine = engine_with_symbols();
        let sidecar = engine.math_sidecar_json().unwrap();
        // save and reopen the page
        let bytes = futures::executor::block_on(engine.save_as_rnote_bytes("p.rnote".into()))
            .unwrap()
            .unwrap();
        let snapshot = futures::executor::block_on(
            crate::engine::EngineSnapshot::load_from_rnote_bytes(bytes),
        )
        .unwrap();
        let mut reopened = Engine::default();
        let _ = reopened.load_snapshot(snapshot);
        assert_eq!(reopened.math.symbols.len(), 0);
        assert_eq!(reopened.math_load_sidecar(&sidecar), 2);
        assert_eq!(reopened.math.overlays.len(), 1);
        // the clean "1" can be corrected again
        let (id, cands) = reopened.math_symbol_at(Vector2::new(101.0, 120.0)).unwrap();
        assert_eq!(cands[0].0, "1");
        let reqs = reopened.math_correction_requests(id, "l", false, None);
        assert_eq!(reqs.len(), 1);
        // new request ids don't collide with the restored ones
        assert!(reopened.math.next_id >= id);
    }

    #[test]
    fn changed_strokes_are_not_restored() {
        let mut engine = engine_with_symbols();
        let sidecar = engine.math_sidecar_json().unwrap();
        // the "=" strokes are erased outside of the math mode
        let eq = engine
            .math
            .symbols
            .iter()
            .find(|s| s.latex == "=")
            .unwrap()
            .keys
            .clone();
        for k in eq {
            engine.store.remove_stroke(k);
        }
        engine.math_reset_document();
        assert_eq!(engine.math_load_sidecar(&sidecar), 1);
        assert_eq!(engine.math_load_sidecar("not json"), 0);
    }

    #[test]
    fn nothing_to_remember() {
        assert!(Engine::default().math_sidecar_json().is_none());
    }
}
