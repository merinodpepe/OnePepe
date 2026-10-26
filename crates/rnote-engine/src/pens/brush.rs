// Imports
use super::PenBehaviour;
use super::PenStyle;
use super::pensconfig::brushconfig::BrushStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::Constraints;
use rnote_compose::Style;
use rnote_compose::builders::buildable::{Buildable, BuilderCreator, BuilderProgress};
use rnote_compose::builders::{
    PenPathBuilderType, PenPathCurvedBuilder, PenPathModeledBuilder, PenPathSimpleBuilder,
};
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::penevent::{PenEvent, PenProgress};
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shaperecognition::{self, RecognitionParams};
use rnote_compose::style::smooth::SmoothOptions;
use std::time::Instant;

#[derive(Debug)]
enum BrushState {
    Idle,
    Drawing {
        path_builder: Box<dyn Buildable<Emit = Segment>>,
        current_stroke_key: StrokeKey,
        preview_style: Style,
    },
}

#[derive(Debug)]
pub struct Brush {
    state: BrushState,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            state: BrushState::Idle,
        }
    }
}

impl PenBehaviour for Brush {
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn style(&self) -> PenStyle {
        PenStyle::Brush
    }

    fn update_state(&mut self, _engine_view: &mut EngineViewMut) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let event_result = match (&mut self.state, event) {
            (BrushState::Idle, PenEvent::Down { element, .. }) => {
                if !element.filter_by_bounds(
                    engine_view
                        .document
                        .bounds()
                        .loosened(Self::INPUT_OVERSHOOT),
                ) {
                    #[cfg(feature = "ui")]
                    {
                        if engine_view.config.pens_config.brush_config.style == BrushStyle::Marker {
                            play_marker_sound(engine_view);
                        } else {
                            trigger_brush_sound(engine_view);
                        }
                    }

                    engine_view
                        .config
                        .pens_config
                        .brush_config
                        .new_style_seeds();

                    let preview_style = Self::get_preview_style(&engine_view.as_im());
                    let brushstroke =
                        Stroke::BrushStroke(BrushStroke::new(element, preview_style.clone()));

                    let current_stroke_key = engine_view.store.insert_stroke(
                        brushstroke,
                        Some(
                            engine_view
                                .config
                                .pens_config
                                .brush_config
                                .layer_for_current_options(),
                        ),
                    );

                    engine_view.store.regenerate_rendering_for_stroke(
                        current_stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    );

                    self.state = BrushState::Drawing {
                        path_builder: new_builder(
                            engine_view.config.pens_config.brush_config.builder_type,
                            element,
                            now,
                        ),
                        current_stroke_key,
                        preview_style,
                    };

                    EventResult {
                        handled: true,
                        propagate: EventPropagation::Stop,
                        progress: PenProgress::InProgress,
                    }
                } else {
                    EventResult {
                        handled: false,
                        propagate: EventPropagation::Proceed,
                        progress: PenProgress::Idle,
                    }
                }
            }
            (BrushState::Idle, _) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (
                BrushState::Drawing {
                    current_stroke_key, ..
                },
                PenEvent::Cancel,
            ) => {
                if let Some(Stroke::BrushStroke(brushstroke)) =
                    engine_view.store.get_stroke_mut(*current_stroke_key)
                {
                    brushstroke.style = engine_view
                        .config
                        .pens_config
                        .brush_config
                        .style_for_current_options();
                }

                // Finish up the last stroke
                engine_view
                    .store
                    .update_geometry_for_stroke(*current_stroke_key);
                engine_view.store.regenerate_rendering_for_stroke_threaded(
                    engine_view.tasks_tx.clone(),
                    *current_stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );
                widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                self.state = BrushState::Idle;

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.store_modified = true;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (
                BrushState::Drawing {
                    path_builder,
                    current_stroke_key,
                    ..
                },
                pen_event,
            ) => {
                let builder_result =
                    path_builder.handle_event(pen_event, now, Constraints::default());
                let handled = builder_result.handled;
                let propagate = builder_result.propagate;

                let progress = match builder_result.progress {
                    BuilderProgress::InProgress => {
                        #[cfg(feature = "ui")]
                        {
                            if engine_view.config.pens_config.brush_config.style
                                != BrushStyle::Marker
                            {
                                trigger_brush_sound(engine_view);
                            }
                        }

                        PenProgress::InProgress
                    }
                    BuilderProgress::EmitContinue(segments) => {
                        #[cfg(feature = "ui")]
                        {
                            if engine_view.config.pens_config.brush_config.style
                                != BrushStyle::Marker
                            {
                                trigger_brush_sound(engine_view);
                            }
                        }

                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.store_modified = true;
                            }

                            engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        PenProgress::InProgress
                    }
                    BuilderProgress::Finished(segments) => {
                        let finished_key = *current_stroke_key;
                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.store_modified = true;
                            }

                            engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        if let Some(Stroke::BrushStroke(brushstroke)) =
                            engine_view.store.get_stroke_mut(*current_stroke_key)
                        {
                            brushstroke.style = engine_view
                                .config
                                .pens_config
                                .brush_config
                                .style_for_current_options();
                        }

                        // Finish up the last stroke
                        engine_view
                            .store
                            .update_geometry_for_stroke(*current_stroke_key);
                        engine_view.store.regenerate_rendering_for_stroke_threaded(
                            engine_view.tasks_tx.clone(),
                            *current_stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        self.state = BrushState::Idle;

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;

                        // OnePepe: ink to shape. The original stroke was recorded above, so undo brings it back.
                        widget_flags |= Self::convert_to_shape(finished_key, engine_view);

                        PenProgress::Finished
                    }
                };

                EventResult {
                    handled,
                    propagate,
                    progress,
                }
            }
        };

        (event_result, widget_flags)
    }
}

impl DrawableOnDoc for Brush {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let style = engine_view
            .config
            .pens_config
            .brush_config
            .style_for_current_options();

        match &self.state {
            BrushState::Idle => None,
            BrushState::Drawing { path_builder, .. } => {
                path_builder.bounds(&style, engine_view.camera.zoom())
            }
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        match &self.state {
            BrushState::Idle => {}
            BrushState::Drawing {
                path_builder,
                preview_style,
                ..
            } => {
                match engine_view.config.pens_config.brush_config.style {
                    BrushStyle::Marker => {
                        // Don't draw the marker, as the pen would render on top of other strokes, while the stroke itself would render underneath them.
                    }
                    BrushStyle::Solid | BrushStyle::Textured => {
                        path_builder.draw_styled(
                            cx,
                            preview_style,
                            engine_view.camera.total_zoom(),
                        );
                    }
                }
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Brush {
    /// OnePepe: minimum size of a stroke to be converted to a shape, in surface pixels.
    const SHAPE_RECOGNITION_MIN_SIZE: f64 = 50.0;

    /// OnePepe: replaces the finished stroke with a geometric shape when the shape recognition is enabled
    /// and the stroke fits one clearly.
    fn convert_to_shape(key: StrokeKey, engine_view: &mut EngineViewMut) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        let brush_config = &engine_view.config.pens_config.brush_config;
        if !brush_config.shape_recognition {
            return widget_flags;
        }
        let Some(Stroke::BrushStroke(brushstroke)) = engine_view.store.get_stroke_ref(key) else {
            return widget_flags;
        };
        let points = brushstroke
            .path
            .clone()
            .into_elements()
            .into_iter()
            .map(|e| e.pos)
            .collect::<Vec<_>>();
        let params = RecognitionParams {
            sensitivity: brush_config.shape_recognition_sensitivity,
            min_size: Self::SHAPE_RECOGNITION_MIN_SIZE / engine_view.camera.total_zoom(),
        };
        let Some(recognized) = shaperecognition::recognize(&points, params) else {
            return widget_flags;
        };
        let mut options = SmoothOptions {
            stroke_width: brushstroke.style.stroke_width(),
            stroke_color: brushstroke.style.stroke_color(),
            fill_color: None,
            ..Default::default()
        };
        options.update_piet_stroke_style();
        let layer = brush_config.layer_for_current_options();

        engine_view.store.remove_stroke(key);
        let new_key = engine_view.store.insert_stroke(
            Stroke::ShapeStroke(ShapeStroke::new(
                recognized.to_shape(),
                Style::Smooth(options),
            )),
            Some(layer),
        );
        engine_view.store.regenerate_rendering_for_stroke(
            new_key,
            engine_view.camera.viewport(),
            engine_view.camera.image_scale(),
        );
        widget_flags |= engine_view.store.record(Instant::now());
        widget_flags.store_modified = true;
        widget_flags.redraw = true;
        widget_flags
    }

    const INPUT_OVERSHOOT: f64 = 30.0;

    fn get_preview_style(engine_view: &EngineView) -> Style {
        let mut style = engine_view
            .config
            .pens_config
            .brush_config
            .style_for_current_options();

        if let Some(mut stroke_color) = style.stroke_color() {
            stroke_color.a = 1.0;
            style.set_stroke_color(stroke_color);
        }

        style
    }
}

#[cfg(feature = "ui")]
fn play_marker_sound(engine_view: &mut EngineViewMut) {
    if let Some(audioplayer) = engine_view.audioplayer {
        audioplayer.play_random_marker_sound();
    }
}

#[cfg(feature = "ui")]
fn trigger_brush_sound(engine_view: &mut EngineViewMut) {
    if let Some(audioplayer) = engine_view.audioplayer.as_mut() {
        audioplayer.trigger_random_brush_sound();
    }
}

fn new_builder(
    builder_type: PenPathBuilderType,
    element: Element,
    now: Instant,
) -> Box<dyn Buildable<Emit = Segment>> {
    match builder_type {
        PenPathBuilderType::Simple => Box::new(PenPathSimpleBuilder::start(element, now)),
        PenPathBuilderType::Curved => Box::new(PenPathCurvedBuilder::start(element, now)),
        PenPathBuilderType::Modeled => Box::new(PenPathModeledBuilder::start(element, now)),
    }
}

#[cfg(test)]
mod tests {
    use crate::Engine;
    use crate::strokes::Stroke;
    use p2d::math::Vector2;
    use rnote_compose::penevent::PenEvent;
    use rnote_compose::penpath::Element;
    use std::collections::HashSet;
    use std::time::{Duration, Instant};

    /// Draws a stroke through the engine like the pen would, returns the kinds of the strokes in the store.
    fn draw(engine: &mut Engine, points: &[Vector2]) -> Vec<&'static str> {
        let t0 = Instant::now();
        for (i, p) in points.iter().enumerate() {
            let element = Element::new(*p, 0.5);
            let event = if i == points.len() - 1 {
                PenEvent::Up {
                    element,
                    modifier_keys: HashSet::new(),
                }
            } else {
                PenEvent::Down {
                    element,
                    modifier_keys: HashSet::new(),
                }
            };
            let _ = engine.handle_pen_event(event, None, t0 + Duration::from_millis(8 * i as u64));
        }
        kinds(engine)
    }

    fn kinds(engine: &Engine) -> Vec<&'static str> {
        engine
            .store
            .stroke_keys_as_rendered()
            .into_iter()
            .filter_map(|k| engine.store.get_stroke_ref(k))
            .map(|s| match s {
                Stroke::BrushStroke(_) => "brush",
                Stroke::ShapeStroke(_) => "shape",
                _ => "other",
            })
            .collect()
    }

    fn wobbly_circle() -> Vec<Vector2> {
        (0..=130)
            .map(|i| {
                let t = i as f64 / 120.0 * 2.0 * std::f64::consts::PI;
                let r = 150.0 + 3.0 * (5.0 * t).sin();
                Vector2::new(400.0 + r * t.cos(), 300.0 + r * t.sin())
            })
            .collect()
    }

    #[test]
    fn ink_to_shape_disabled_by_default() {
        let mut engine = Engine::default();
        assert_eq!(draw(&mut engine, &wobbly_circle()), vec!["brush"]);
    }

    #[test]
    fn ink_to_shape_converts_and_undo_restores_the_original() {
        let mut engine = Engine::default();
        engine
            .config
            .write()
            .pens_config
            .brush_config
            .shape_recognition = true;
        assert_eq!(draw(&mut engine, &wobbly_circle()), vec!["shape"]);
        let _ = engine.undo(Instant::now());
        assert_eq!(kinds(&engine), vec!["brush"]);
        let _ = engine.redo(Instant::now());
        assert_eq!(kinds(&engine), vec!["shape"]);
    }

    #[test]
    fn ink_to_shape_keeps_handwriting() {
        let mut engine = Engine::default();
        engine
            .config
            .write()
            .pens_config
            .brush_config
            .shape_recognition = true;
        // a big cursive "e" loop chain
        let pts = (0..300)
            .map(|i| {
                let t = i as f64 / 300.0 * 6.0 * std::f64::consts::PI;
                Vector2::new(t * 20.0 - 40.0 * t.sin(), -40.0 * t.cos())
            })
            .collect::<Vec<Vector2>>();
        assert_eq!(draw(&mut engine, &pts), vec!["brush"]);
    }
}
