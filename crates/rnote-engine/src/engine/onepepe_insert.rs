// OnePepe: things to insert into the page (tables, templates, title and date), the page size and the export
// that fits the drawing on the page size.

// Imports
use super::{Engine, StrokeContent};
use crate::WidgetFlags;
use crate::pens::PenStyle;
use crate::store::StrokeKey;
use crate::strokes::textstroke::TextAttribute;
use crate::strokes::{ShapeStroke, Stroke, TextStroke};
use anyhow::Context;
use p2d::bounding_volume::Aabb;
use p2d::math::Vector2;
use rnote_compose::Color;
use rnote_compose::shapes::{Line, Shape};
use rnote_compose::style::Style;
use rnote_compose::style::smooth::SmoothOptions;
use std::time::Instant;

/// Paper sizes (DIN and US Letter).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    A5,
    A4,
    A3,
    A2,
    A1,
    A0,
    Letter,
}

impl PageSize {
    /// Width and height in millimeters, portrait.
    pub fn size_mm(self) -> (f64, f64) {
        match self {
            PageSize::A5 => (148.0, 210.0),
            PageSize::A4 => (210.0, 297.0),
            PageSize::A3 => (297.0, 420.0),
            PageSize::A2 => (420.0, 594.0),
            PageSize::A1 => (594.0, 841.0),
            PageSize::A0 => (841.0, 1189.0),
            PageSize::Letter => (215.9, 279.4),
        }
    }
}

impl PageSize {
    pub const ALL: [PageSize; 7] = [
        PageSize::A5,
        PageSize::A4,
        PageSize::A3,
        PageSize::A2,
        PageSize::A1,
        PageSize::A0,
        PageSize::Letter,
    ];

    /// The name shown to the user.
    pub fn name(self) -> &'static str {
        match self {
            PageSize::A5 => "A5",
            PageSize::A4 => "A4",
            PageSize::A3 => "A3",
            PageSize::A2 => "A2",
            PageSize::A1 => "A1",
            PageSize::A0 => "A0",
            PageSize::Letter => "Carta",
        }
    }

    /// The size that matches a page of `width` × `height` millimeters (1 mm of margin), and whether it is landscape.
    pub fn matching(width: f64, height: f64) -> Option<(PageSize, bool)> {
        Self::ALL.into_iter().find_map(|size| {
            let (w, h) = size.size_mm();
            if (w - width).abs() < 1.0 && (h - height).abs() < 1.0 {
                Some((size, false))
            } else if (h - width).abs() < 1.0 && (w - height).abs() < 1.0 {
                Some((size, true))
            } else {
                None
            }
        })
    }
}

/// OnePepe: what the status bar shows about a document.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DocStatus {
    pub strokes: usize,
    pub pages: usize,
    /// The page size, if it is a standard one, and whether it is landscape.
    pub page_size: Option<(PageSize, bool)>,
    pub zoom: f64,
}

impl std::str::FromStr for PageSize {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a5" => Ok(Self::A5),
            "a4" => Ok(Self::A4),
            "a3" => Ok(Self::A3),
            "a2" => Ok(Self::A2),
            "a1" => Ok(Self::A1),
            "a0" => Ok(Self::A0),
            "letter" => Ok(Self::Letter),
            _ => Err(anyhow::anyhow!("unknown page size '{s}'")),
        }
    }
}

/// Page templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTemplate {
    /// Cornell notes: title band, cue column on the left, summary at the bottom.
    Cornell,
    /// Class notes: title, date and a line below.
    ClassNotes,
    /// Problem sheet: title, date and the page split in two columns.
    Problems,
}

impl std::str::FromStr for PageTemplate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cornell" => Ok(Self::Cornell),
            "class-notes" => Ok(Self::ClassNotes),
            "problems" => Ok(Self::Problems),
            _ => Err(anyhow::anyhow!("unknown page template '{s}'")),
        }
    }
}

/// How the export fits the drawing on the page size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitMode {
    /// Everything shrunk onto a single page.
    OnePage,
    /// Shrunk to the page width, continued on as many pages as needed.
    PageWidth,
}

/// The scale (at most 1, never enlarges) that fits `content` into `page` keeping a margin of `margin` on every side.
pub fn fit_scale(content: Vector2, page: Vector2, margin: f64, mode: FitMode) -> f64 {
    let avail = Vector2::new(
        (page[0] - 2.0 * margin).max(1.0),
        (page[1] - 2.0 * margin).max(1.0),
    );
    let scale = match mode {
        FitMode::OnePage => (avail[0] / content[0]).min(avail[1] / content[1]),
        FitMode::PageWidth => avail[0] / content[0],
    };
    scale.min(1.0)
}

/// The lines of a table with `rows` x `cols` cells of `cell` size, upper left corner at `origin`.
pub fn table_lines(origin: Vector2, rows: u32, cols: u32, cell: Vector2) -> Vec<Line> {
    let size = Vector2::new(cell[0] * cols as f64, cell[1] * rows as f64);
    let mut lines = Vec::new();
    for r in 0..=rows {
        let y = origin[1] + cell[1] * r as f64;
        lines.push(Line {
            start: Vector2::new(origin[0], y),
            end: Vector2::new(origin[0] + size[0], y),
        });
    }
    for c in 0..=cols {
        let x = origin[0] + cell[0] * c as f64;
        lines.push(Line {
            start: Vector2::new(x, origin[1]),
            end: Vector2::new(x, origin[1] + size[1]),
        });
    }
    lines
}

/// The lines of a page template, for a page with the given bounds. Positions are relative to the page.
pub fn template_lines(template: PageTemplate, page: Aabb) -> Vec<Line> {
    let (x0, y0) = (page.mins[0], page.mins[1]);
    let (w, h) = (page.extents()[0], page.extents()[1]);
    let margin = w * 0.05;
    let title_band = h * 0.08;
    let hline = |y: f64| Line {
        start: Vector2::new(x0 + margin, y0 + y),
        end: Vector2::new(x0 + w - margin, y0 + y),
    };
    match template {
        PageTemplate::Cornell => vec![
            hline(title_band),
            hline(h * 0.78),
            Line {
                start: Vector2::new(x0 + w * 0.3, y0 + title_band),
                end: Vector2::new(x0 + w * 0.3, y0 + h * 0.78),
            },
        ],
        PageTemplate::ClassNotes => vec![hline(title_band)],
        PageTemplate::Problems => vec![
            hline(title_band),
            Line {
                start: Vector2::new(x0 + w * 0.5, y0 + title_band),
                end: Vector2::new(x0 + w * 0.5, y0 + h - margin),
            },
        ],
    }
}

impl Engine {
    /// Sets the page size of the document (DIN A5 to A0, Letter), portrait or landscape.
    pub fn doc_status(&self) -> DocStatus {
        let format = &self.document.config.format;
        let to_mm = |px: f64| px / format.dpi() * 25.4;
        let pages_down = (self.document.config.layout == crate::document::Layout::FixedSize
            || self.document.config.layout == crate::document::Layout::ContinuousVertical)
            .then(|| (self.document.height / format.height()).ceil().max(1.0) as usize);
        DocStatus {
            strokes: self.store.stroke_keys_unordered().len(),
            pages: pages_down.unwrap_or(1),
            page_size: PageSize::matching(to_mm(format.width()), to_mm(format.height())),
            zoom: self.camera.zoom(),
        }
    }

    pub fn set_page_size(&mut self, size: PageSize, landscape: bool) -> WidgetFlags {
        let (mut w_mm, mut h_mm) = size.size_mm();
        if landscape {
            std::mem::swap(&mut w_mm, &mut h_mm);
        }
        let dpi = self.document.config.format.dpi();
        let to_px = |mm: f64| mm / 25.4 * dpi;
        self.document.config.format.set_width(to_px(w_mm));
        self.document.config.format.set_height(to_px(h_mm));
        let mut widget_flags = self.doc_resize_to_fit_content();
        widget_flags.store_modified = true;
        widget_flags.refresh_ui = true;
        widget_flags
    }

    /// The bounds of the page under the center of the view.
    fn page_bounds_at_view_center(&self) -> Aabb {
        let format = self.document.config.format.size();
        let center = self.camera.viewport().center();
        let origin = Vector2::ZERO;
        let col = ((center[0] - origin[0]) / format[0]).floor();
        let row = ((center[1] - origin[1]) / format[1]).floor();
        let mins = origin + Vector2::new(col * format[0], row * format[1]);
        Aabb::new(mins, mins + format)
    }

    fn line_stroke(line: Line, width: f64) -> Stroke {
        let mut options = SmoothOptions {
            stroke_width: width,
            stroke_color: Some(Color::new(0.2, 0.2, 0.2, 1.0)),
            fill_color: None,
            ..Default::default()
        };
        options.update_piet_stroke_style();
        Stroke::ShapeStroke(ShapeStroke::new(Shape::Line(line), Style::Smooth(options)))
    }

    /// Inserts the strokes, selects them (so they can be moved right away) and records the change for undo.
    fn insert_and_select(&mut self, strokes: Vec<Stroke>, select: bool) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        let all_strokes = self.store.stroke_keys_as_rendered();
        self.store.set_selected_keys(&all_strokes, false);
        let keys: Vec<StrokeKey> = strokes
            .into_iter()
            .map(|stroke| self.store.insert_stroke(stroke, None))
            .collect();
        self.store.update_geometry_for_strokes(&keys);
        for key in &keys {
            self.store.regenerate_rendering_for_stroke(
                *key,
                self.camera.viewport(),
                self.camera.image_scale(),
            );
        }
        if select {
            widget_flags |= self.change_pen_style(PenStyle::Selector);
            self.store.set_selected_keys(&keys, true);
            widget_flags |= self.current_pen_update_state();
        }
        widget_flags |= self.doc_resize_autoexpand();
        widget_flags |= self.record(Instant::now());
        widget_flags.store_modified = true;
        widget_flags.redraw = true;
        widget_flags.resize = true;
        widget_flags
    }

    /// Inserts a table of `rows` x `cols` empty cells drawn with lines, in the middle of the view, selected.
    /// The cells are made to be written in by hand.
    pub fn insert_table(&mut self, rows: u32, cols: u32) -> WidgetFlags {
        let rows = rows.clamp(1, 30);
        let cols = cols.clamp(1, 15);
        let dpi = self.document.config.format.dpi();
        // cells of 35 mm x 12 mm
        let cell = Vector2::new(35.0 / 25.4 * dpi, 12.0 / 25.4 * dpi);
        let size = Vector2::new(cell[0] * cols as f64, cell[1] * rows as f64);
        let origin = self.camera.viewport().center() - size * 0.5;
        let strokes = table_lines(origin, rows, cols, cell)
            .into_iter()
            .map(|line| Self::line_stroke(line, 1.5))
            .collect();
        self.insert_and_select(strokes, true)
    }

    fn title_text_stroke(&self, text: String, pos: Vector2, scale: f64, bold: bool) -> Stroke {
        let mut text_style = self
            .config
            .read()
            .pens_config
            .typewriter_config
            .text_style
            .clone();
        text_style.font_size = (text_style.font_size * scale).round();
        let len = text.len();
        let mut textstroke = TextStroke::new(text, pos, text_style);
        if bold && len > 0 {
            textstroke.replace_attr_for_range(0..len, TextAttribute::FontWeight(700));
        }
        Stroke::TextStroke(textstroke)
    }

    /// Inserts the title (big and bold) and today's date at the top of the page under the view, like the page
    /// header of OneNote.
    pub fn insert_title_and_date(&mut self, title: &str) -> WidgetFlags {
        let page = self.page_bounds_at_view_center();
        let margin = page.extents()[0] * 0.05;
        let pos = Vector2::new(page.mins[0] + margin, page.mins[1] + margin * 0.4);
        let title = if title.trim().is_empty() {
            "Título".to_string()
        } else {
            title.to_string()
        };
        let date = crate::pens::typewriter::scripts::date_time_stamp(
            crate::pens::typewriter::scripts::DateTimeStamp::DateAndTime,
            chrono::Local::now().naive_local(),
        );
        let font_size = self
            .config
            .read()
            .pens_config
            .typewriter_config
            .text_style
            .font_size;
        let strokes = vec![
            self.title_text_stroke(title, pos, 1.8, true),
            self.title_text_stroke(date, pos + Vector2::new(0.0, font_size * 2.6), 0.8, false),
        ];
        self.insert_and_select(strokes, false)
    }

    /// Inserts a page template (lines, title and date) on the page under the view.
    pub fn insert_page_template(&mut self, template: PageTemplate, title: &str) -> WidgetFlags {
        let page = self.page_bounds_at_view_center();
        let strokes = template_lines(template, page)
            .into_iter()
            .map(|line| Self::line_stroke(line, 1.5))
            .collect();
        let widget_flags = self.insert_and_select(strokes, false);
        widget_flags | self.insert_title_and_date(title)
    }

    /// Exports the drawing as PDF on pages of the document page size, shrunk to fit (see [FitMode]).
    ///
    /// Returns the PDF bytes and the scale that was applied (1.0 = real size), so the caller can warn when the
    /// text becomes too small.
    pub fn export_doc_as_pdf_fitted(
        &self,
        title: String,
        mode: FitMode,
    ) -> anyhow::Result<(Vec<u8>, f64)> {
        const PT_IN_INCH: f64 = 72.0;
        let prefs = self.config.read().export_prefs.doc_export_prefs;
        let content: StrokeContent = self
            .store
            .fetch_stroke_content(&self.store.stroke_keys_as_rendered())
            .with_background(self.document.config.background);
        let Some(bounds) = content.bounds() else {
            return Err(anyhow::anyhow!("The page is empty, nothing to export."));
        };
        let page = self.document.config.format.size();
        let dpi = self.document.config.format.dpi();
        let margin = page[0] * 0.04;
        let scale = fit_scale(bounds.extents(), page, margin, mode);
        let n_pages = match mode {
            FitMode::OnePage => 1,
            FitMode::PageWidth => ((bounds.extents()[1] * scale) / (page[1] - 2.0 * margin))
                .ceil()
                .max(1.0) as usize,
        };
        let page_content_height = (page[1] - 2.0 * margin) / scale;

        let surface = cairo::PdfSurface::for_stream(
            page[0] * (PT_IN_INCH / dpi),
            page[1] * (PT_IN_INCH / dpi),
            Vec::<u8>::new(),
        )
        .context("Creating Pdf target surface failed.")?;
        surface
            .set_metadata(cairo::PdfMetadata::Title, title.as_str())
            .context("Set pdf surface title metadata failed.")?;
        {
            let cx = cairo::Context::new(&surface).context("Creating cairo context failed.")?;
            cx.scale(PT_IN_INCH / dpi, PT_IN_INCH / dpi);
            for i in 0..n_pages {
                let slice_top = bounds.mins[1] + page_content_height * i as f64;
                let slice = Aabb::new(
                    Vector2::new(bounds.mins[0], slice_top),
                    Vector2::new(
                        bounds.maxs[0],
                        (slice_top + page_content_height).min(bounds.maxs[1]),
                    ),
                );
                let slice_content = content.clone().with_bounds(slice);
                cx.save()?;
                // center horizontally, top aligned
                let offset_x = (page[0] - bounds.extents()[0] * scale) * 0.5;
                cx.translate(offset_x, margin);
                cx.scale(scale, scale);
                cx.translate(-slice.mins[0], -slice.mins[1]);
                slice_content.draw_to_cairo(
                    &cx,
                    prefs.with_background,
                    prefs.with_pattern,
                    prefs.optimize_printing,
                    0.0,
                    Engine::STROKE_EXPORT_IMAGE_SCALE,
                )?;
                cx.restore()?;
                cx.show_page()
                    .map_err(|e| anyhow::anyhow!("Showing pdf page {i} failed, Err: {e:?}"))?;
            }
        }
        let data = *surface
            .finish_output_stream()
            .map_err(|e| anyhow::anyhow!("Finishing outputstream failed, Err: {e:?}"))?
            .downcast::<Vec<u8>>()
            .map_err(|e| anyhow::anyhow!("Downcasting output stream failed, Err: {e:?}"))?;
        Ok((data, scale))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn page_size_matching() {
        use super::PageSize;
        assert_eq!(
            PageSize::matching(210.0, 297.0),
            Some((PageSize::A4, false))
        );
        assert_eq!(PageSize::matching(297.2, 210.0), Some((PageSize::A4, true)));
        assert_eq!(PageSize::matching(100.0, 100.0), None);
    }

    use super::*;

    #[test]
    fn fit_scale_shrinks_never_enlarges() {
        let a4 = Vector2::new(210.0, 297.0);
        let a0 = Vector2::new(841.0, 1189.0);
        let s = fit_scale(a0, a4, 0.0, FitMode::OnePage);
        assert!((s - 0.2497).abs() < 0.001, "{s}");
        assert_eq!(
            fit_scale(Vector2::new(10.0, 10.0), a4, 0.0, FitMode::OnePage),
            1.0
        );
        // wide and long drawing: page width mode only looks at the width
        let s = fit_scale(Vector2::new(420.0, 3000.0), a4, 0.0, FitMode::PageWidth);
        assert!((s - 0.5).abs() < 1e-9);
    }

    #[test]
    fn table_has_all_lines() {
        let lines = table_lines(Vector2::ZERO, 3, 4, Vector2::new(10.0, 5.0));
        assert_eq!(lines.len(), (3 + 1) + (4 + 1));
        assert_eq!(lines[3].end, Vector2::new(40.0, 15.0));
    }

    #[test]
    fn templates_stay_inside_the_page() {
        let page = Aabb::new(Vector2::new(100.0, 200.0), Vector2::new(900.0, 1300.0));
        for t in [
            PageTemplate::Cornell,
            PageTemplate::ClassNotes,
            PageTemplate::Problems,
        ] {
            for l in template_lines(t, page) {
                for p in [l.start, l.end] {
                    assert!(page.contains_local_point(p), "{t:?} {p:?}");
                }
            }
        }
        assert_eq!("a1".parse::<PageSize>().unwrap().size_mm(), (594.0, 841.0));
    }

    #[test]
    fn insert_table_and_template_record_strokes() {
        let mut engine = Engine::default();
        let _ = engine.insert_table(2, 3);
        assert_eq!(engine.store.stroke_keys_as_rendered().len(), 3 + 4);
        assert_eq!(engine.store.selection_keys_as_rendered().len(), 7);
        let _ = engine.insert_page_template(PageTemplate::Cornell, "Biofísica");
        assert_eq!(engine.store.stroke_keys_as_rendered().len(), 7 + 3 + 2);
        let _ = engine.undo(Instant::now());
        assert_eq!(engine.store.stroke_keys_as_rendered().len(), 7 + 3);
        let _ = engine.set_page_size(PageSize::A4, true);
        let size = engine.document.config.format.size();
        assert!(size[0] > size[1]);
        let (pdf, scale) = engine
            .export_doc_as_pdf_fitted("t".into(), FitMode::OnePage)
            .unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(scale <= 1.0);
    }
}
