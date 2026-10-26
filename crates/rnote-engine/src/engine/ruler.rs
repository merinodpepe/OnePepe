//! OnePepe: integration of the virtual ruler into the engine.

// Imports
use super::Engine;
use crate::WidgetFlags;
use crate::engine_view;
use crate::pens::PenStyle;
use p2d::math::Vector2;
use rnote_compose::penevent::PenEvent;
use rnote_compose::ruler::{Ruler, RulerInteraction};

impl Engine {
    /// How close to an edge (in surface pixels) the pen must start to be snapped.
    const RULER_SNAP_DISTANCE: f64 = 16.0;
    /// Width of a new ruler, in surface pixels.
    const RULER_WIDTH: f64 = 72.0;

    /// Whether the ruler is shown.
    pub fn ruler_enabled(&self) -> bool {
        self.ruler.is_some()
    }

    /// Shows or hides the ruler. A new ruler is placed horizontally in the middle of the current view.
    pub fn set_ruler_enabled(&mut self, enabled: bool) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if enabled == self.ruler_enabled() {
            return widget_flags;
        }
        self.ruler_interaction = RulerInteraction::Idle;
        self.ruler = if enabled {
            let viewport = self.camera.viewport();
            Some(Ruler {
                center: viewport.center(),
                angle: 0.0,
                length: viewport.extents().x * 0.7,
                width: Self::RULER_WIDTH / self.camera.total_zoom(),
            })
        } else {
            None
        };
        widget_flags.redraw = true;
        widget_flags
    }

    /// Applies the ruler to a pen event.
    ///
    /// Returns the (possibly snapped) event that should be passed on to the pen,
    /// or `None` when the event was used to move or rotate the ruler.
    pub(crate) fn apply_ruler_to_pen_event(&mut self, event: PenEvent) -> Option<PenEvent> {
        let Some(ruler) = self.ruler.as_mut() else {
            return Some(event);
        };
        // only the brush uses the ruler
        if self
            .penholder
            .current_pen_style_w_override(&engine_view!(self))
            != PenStyle::Brush
        {
            self.ruler_interaction = RulerInteraction::Idle;
            return Some(event);
        }
        let snap_distance = Self::RULER_SNAP_DISTANCE / self.camera.total_zoom();

        match event {
            PenEvent::Down {
                mut element,
                modifier_keys,
            } => {
                element.pos = self
                    .ruler_interaction
                    .pen_down(ruler, element.pos, snap_distance)?;
                Some(PenEvent::Down {
                    element,
                    modifier_keys,
                })
            }
            PenEvent::Up {
                mut element,
                modifier_keys,
            } => {
                element.pos = self
                    .ruler_interaction
                    .pen_up(ruler, element.pos, snap_distance)?;
                Some(PenEvent::Up {
                    element,
                    modifier_keys,
                })
            }
            PenEvent::Cancel => {
                self.ruler_interaction = RulerInteraction::Idle;
                Some(event)
            }
            other => Some(other),
        }
    }

    /// Draws the ruler on top of the document.
    #[cfg(feature = "ui")]
    pub(crate) fn draw_ruler_to_gtk_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
    ) -> anyhow::Result<()> {
        use crate::ext::GrapheneRectExt;
        use gtk4::{graphene, prelude::*};
        use p2d::bounding_volume::{Aabb, BoundingVolume};
        use piet::RenderContext;
        use rnote_compose::ext::{AabbExt, DAffine2Ext};

        let Some(ruler) = self.ruler else {
            return Ok(());
        };
        let total_zoom = self.camera.total_zoom();
        let corners = ruler.corners();
        let mins = corners.iter().fold(corners[0], |acc, c| acc.min(*c));
        let maxs = corners.iter().fold(corners[0], |acc, c| acc.max(*c));
        let viewport = self.camera.viewport();
        if !Aabb::new(mins, maxs).intersects(&viewport) {
            return Ok(());
        }
        let bounds = Aabb::new(mins, maxs).clamp(None, Some(viewport));
        let mut bounds_on_surface = bounds
            .scale(total_zoom)
            .translate(-self.camera.offset())
            .ceil();
        bounds_on_surface.ensure_positive();
        bounds_on_surface.assert_valid()?;

        snapshot.save();
        let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(bounds_on_surface));
        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);
        piet_cx.transform(self.camera.transform().to_kurbo());

        draw_ruler(&mut piet_cx, &ruler, total_zoom)?;

        piet_cx.finish().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        snapshot.restore();
        Ok(())
    }
}

/// Draws the ruler with a piet render context in document coordinates.
pub fn draw_ruler(
    piet_cx: &mut impl piet::RenderContext,
    ruler: &Ruler,
    total_zoom: f64,
) -> anyhow::Result<()> {
    use piet::{Text, TextLayoutBuilder};
    use rnote_compose::color;
    use rnote_compose::ext::Vector2Ext;

    let corners = ruler.corners();
    let line_width = 1.5 / total_zoom;
    let fill_color = color::GNOME_BRIGHTS[1].with_a8(200);
    let edge_color = color::GNOME_DARKS[2].with_a8(230);

    // body
    let mut body = kurbo::BezPath::new();
    body.move_to(corners[0].to_kurbo_point());
    for c in &corners[1..] {
        body.line_to(c.to_kurbo_point());
    }
    body.close_path();
    piet_cx.fill(&body, &fill_color);
    piet_cx.stroke(&body, &edge_color, line_width);

    // ticks on both edges, every 10 units, longer every 50 and 100
    let half_len = ruler.length * 0.5;
    let half_width = ruler.width * 0.5;
    let mut i = 0;
    loop {
        let along = -half_len + f64::from(i) * 10.0;
        if along > half_len {
            break;
        }
        let tick_len = if i % 10 == 0 {
            0.3
        } else if i % 5 == 0 {
            0.2
        } else {
            0.1
        } * ruler.width;
        for (edge, dir) in [(-half_width, 1.0), (half_width, -1.0)] {
            let p0 = ruler.from_local(Vector2::new(along, edge));
            let p1 = ruler.from_local(Vector2::new(along, edge + dir * tick_len));
            piet_cx.stroke(
                kurbo::Line::new(p0.to_kurbo_point(), p1.to_kurbo_point()),
                &edge_color,
                line_width * 0.7,
            );
        }
        i += 1;
    }

    // rotation handle
    piet_cx.stroke(
        kurbo::Circle::new(
            ruler.rotate_handle_center().to_kurbo_point(),
            ruler.rotate_handle_radius(),
        ),
        &color::GNOME_BLUES[3],
        line_width * 1.5,
    );

    // angle
    let angle_text = format!("{:.0}°", ruler.angle_degrees());
    let font_size = ruler.width * 0.25;
    if let Ok(layout) = piet_cx
        .text()
        .new_text_layout(angle_text)
        .text_color(color::GNOME_DARKS[3])
        .font(piet::FontFamily::SANS_SERIF, font_size)
        .build()
    {
        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let text_pos = ruler.from_local(Vector2::new(-font_size, -font_size * 0.6));
        piet_cx.transform(
            kurbo::Affine::translate(text_pos.to_kurbo_vec()) * kurbo::Affine::rotate(ruler.angle),
        );
        piet_cx.draw_text(&layout, kurbo::Point::ZERO);
        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rnote_compose::penpath::Element;
    use std::collections::HashSet;

    fn down(x: f64, y: f64) -> PenEvent {
        PenEvent::Down {
            element: Element::new(Vector2::new(x, y), 0.5),
            modifier_keys: HashSet::new(),
        }
    }

    #[test]
    fn engine_snaps_brush_events_to_the_ruler() {
        let mut engine = Engine::default();
        let _ = engine.camera_set_size(Vector2::new(1000.0, 800.0));
        let _ = engine.set_ruler_enabled(true);
        let ruler = engine.ruler.unwrap();
        let edge_y = ruler.center.y + ruler.width * 0.5;

        let ev = engine
            .apply_ruler_to_pen_event(down(ruler.center.x, edge_y + 5.0))
            .unwrap();
        let PenEvent::Down { element, .. } = ev else {
            panic!("expected a down event")
        };
        assert!((element.pos.y - edge_y).abs() < 1e-9);

        // the body moves the ruler and the event is consumed
        let mut engine2 = Engine::default();
        let _ = engine2.camera_set_size(Vector2::new(1000.0, 800.0));
        let _ = engine2.set_ruler_enabled(true);
        let c = engine2.ruler.unwrap().center;
        assert!(engine2.apply_ruler_to_pen_event(down(c.x, c.y)).is_none());
        assert!(
            engine2
                .apply_ruler_to_pen_event(down(c.x + 10.0, c.y))
                .is_none()
        );
        assert!((engine2.ruler.unwrap().center.x - (c.x + 10.0)).abs() < 1e-9);

        let _ = engine.set_ruler_enabled(false);
        assert!(engine.ruler.is_none());
    }

    #[test]
    fn ruler_draws_something() {
        let ruler = Ruler {
            center: Vector2::new(200.0, 100.0),
            angle: -30_f64.to_radians(),
            length: 340.0,
            width: 60.0,
        };
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 400, 200).unwrap();
        {
            let cx = cairo::Context::new(&surface).unwrap();
            cx.set_source_rgb(1.0, 1.0, 1.0);
            cx.paint().unwrap();
            let mut piet_cx = piet_cairo::CairoRenderContext::new(&cx);
            draw_ruler(&mut piet_cx, &ruler, 1.0).unwrap();
            piet::RenderContext::finish(&mut piet_cx).unwrap();
        }
        if let Ok(path) = std::env::var("ONEPEPE_RENDER_RULER") {
            let mut file = std::fs::File::create(path).unwrap();
            surface.write_to_png(&mut file).unwrap();
        }
        let data = surface.take_data().unwrap();
        let not_white = data
            .chunks(4)
            .filter(|px| px[0] < 200 || px[1] < 200)
            .count();
        assert!(
            not_white > 500,
            "the ruler was not drawn ({not_white} pixels)"
        );
    }
}
