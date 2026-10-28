// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the favorite pens of the ribbon (Draw tab), as in the design guide: up to 5 slots, 28 px circles that
// preview the stroke, a brand ring on the active pen.
// Click: choose it. Hold, or «+»: save the current pen (in that slot, or in the first free one). Right click:
// save here, move, delete. Drag: reorder. Ctrl+1…Ctrl+5: choose. Saved in ~/.config/onepepe/boligrafos.json.

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gdk, gio, glib, glib::clone, prelude::*};
use rnote_engine::pens::PenStyle;
use rnote_engine::pens::penpresets::{PenPreset, PenPresets};
use rnote_engine::pens::pensconfig::brushconfig::BrushStyle;
use std::cell::RefCell;
use tracing::error;

thread_local! {
    /// The favorite pens, loaded from the settings file.
    static PEN_PRESETS: RefCell<Option<PenPresets>> = const { RefCell::new(None) };
    /// The box of the ribbon that shows the favorite pens.
    static FAVORITES_BOX: RefCell<Option<glib::WeakRef<gtk4::Box>>> = const { RefCell::new(None) };
}

fn presets_path() -> std::path::PathBuf {
    glib::user_config_dir()
        .join("onepepe")
        .join("boligrafos.json")
}

fn with_presets<R>(f: impl FnOnce(&mut PenPresets) -> R) -> R {
    PEN_PRESETS.with(|p| {
        let mut p = p.borrow_mut();
        let presets = p.get_or_insert_with(|| {
            let Ok(json) = std::fs::read_to_string(presets_path()) else {
                return PenPresets::default();
            };
            let mut presets = PenPresets::from_json(&json);
            if presets.fit_slots() {
                // a list from before the slots: keep it whole next to the new one
                let _ = std::fs::write(presets_path().with_extension("anterior.json"), &json);
            }
            presets
        });
        f(presets)
    })
}

fn save_presets() {
    let json = with_presets(|p| p.to_json());
    let path = presets_path();
    let result = json.and_then(|json| {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, json)?;
        Ok(())
    });
    if let Err(e) = result {
        error!("Saving the favorite pens failed, Err: {e:?}");
    }
}

/// The box with the favorite pens, for the ribbon.
pub(crate) fn widget(appwindow: &RnAppWindow) -> gtk4::Box {
    let favorites = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    favorites.set_valign(gtk4::Align::Center);
    favorites.add_css_class("onepepe-favorites");
    FAVORITES_BOX.with(|b| *b.borrow_mut() = Some(favorites.downgrade()));
    rebuild(appwindow);
    favorites
}

/// A 28 px circle that previews the stroke: a short wave in the color and (roughly) the width of the pen.
fn swatch(preset: &PenPreset) -> gtk4::DrawingArea {
    let area = gtk4::DrawingArea::builder()
        .content_width(28)
        .content_height(28)
        .build();
    area.add_css_class("onepepe-pen-swatch");
    let color = preset.color;
    let marker = matches!(preset.style, BrushStyle::Marker);
    let width = if marker {
        8.0
    } else {
        (preset.width * 0.8).clamp(1.5, 7.0)
    };
    area.set_draw_func(move |area, cx, w, h| {
        let (w, h) = (w as f64, h as f64);
        // the circle, in the surface color of the theme
        let surface = gdk::RGBA::parse(
            crate::theme::token("bg_raised", crate::theme::is_dark(), false).unwrap_or("#FFFFFF"),
        )
        .unwrap_or(gdk::RGBA::WHITE);
        let border = area.color();
        cx.arc(w / 2.0, h / 2.0, w / 2.0 - 1.0, 0.0, std::f64::consts::TAU);
        cx.set_source_rgba(
            surface.red() as f64,
            surface.green() as f64,
            surface.blue() as f64,
            1.0,
        );
        let _ = cx.fill_preserve();
        cx.set_source_rgba(
            border.red() as f64,
            border.green() as f64,
            border.blue() as f64,
            0.25,
        );
        cx.set_line_width(1.0);
        let _ = cx.stroke();
        // the stroke
        cx.arc(w / 2.0, h / 2.0, w / 2.0 - 2.0, 0.0, std::f64::consts::TAU);
        cx.clip();
        cx.move_to(w * 0.18, h * 0.62);
        cx.curve_to(w * 0.38, h * 0.18, w * 0.62, h * 0.82, w * 0.82, h * 0.38);
        cx.set_source_rgba(
            color.r,
            color.g,
            color.b,
            if marker { color.a * 0.6 } else { color.a },
        );
        cx.set_line_width(width);
        cx.set_line_cap(gtk4::cairo::LineCap::Round);
        let _ = cx.stroke();
    });
    area
}

/// The tooltip of a pen.
fn describe(preset: &PenPreset, slot: usize) -> String {
    let kind = match preset.style {
        BrushStyle::Marker => gettext("Highlighter"),
        BrushStyle::Solid => gettext("Pen"),
        BrushStyle::Textured => gettext("Pencil"),
    };
    format!(
        "{} · {} {:.1} · Ctrl+{}\n{}",
        preset.name,
        kind,
        preset.width,
        slot + 1,
        gettext("Hold or right click to save the current pen here")
    )
}

pub(crate) fn rebuild(appwindow: &RnAppWindow) {
    let Some(container) = FAVORITES_BOX.with(|b| b.borrow().as_ref().and_then(|w| w.upgrade()))
    else {
        return;
    };
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    let presets = with_presets(|p| p.0.clone());
    for (i, preset) in presets.iter().enumerate() {
        let button = gtk4::Button::builder()
            .child(&swatch(preset))
            .tooltip_text(describe(preset, i))
            .action_name("win.pen-preset")
            .action_target(&(i as i32).to_variant())
            .build();
        button.add_css_class("flat");
        button.add_css_class("onepepe-pen");
        button.set_focus_on_click(false);

        // right click: the menu of the slot
        let right_click = gtk4::GestureClick::builder().button(3).build();
        right_click.connect_pressed(clone!(
            #[weak]
            button,
            #[weak]
            appwindow,
            move |_, _, _, _| slot_menu(&appwindow, &button, i, presets_len())
        ));
        button.add_controller(right_click);

        // hold: save the current pen in this slot
        let hold = gtk4::GestureLongPress::new();
        hold.connect_pressed(clone!(
            #[weak]
            appwindow,
            move |gesture, _, _| {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                activate(&appwindow, "pen-preset-save-at", i as i32);
            }
        ));
        button.add_controller(hold);

        // drag and drop to reorder
        let drag = gtk4::DragSource::builder()
            .actions(gdk::DragAction::MOVE)
            .content(&gdk::ContentProvider::for_value(&(i as i32).to_value()))
            .build();
        button.add_controller(drag);
        let drop = gtk4::DropTarget::new(i32::static_type(), gdk::DragAction::MOVE);
        drop.connect_drop(clone!(
            #[weak]
            appwindow,
            #[upgrade_or]
            false,
            move |_, value, _, _| {
                let Ok(from) = value.get::<i32>() else {
                    return false;
                };
                with_presets(|p| p.move_to(from as usize, i));
                save_presets();
                rebuild(&appwindow);
                true
            }
        ));
        button.add_controller(drop);
        container.append(&button);
    }
    if presets.len() < PenPresets::MAX {
        let add = gtk4::Button::builder()
            .icon_name("plus-symbolic")
            .tooltip_text(gettext("Save the current pen as favorite"))
            .action_name("win.pen-preset-save")
            .valign(gtk4::Align::Center)
            .build();
        add.add_css_class("flat");
        add.add_css_class("circular");
        add.set_focus_on_click(false);
        container.append(&add);
    }
    refresh(appwindow);
}

fn presets_len() -> usize {
    with_presets(|p| p.0.len())
}

fn activate(appwindow: &RnAppWindow, action: &str, index: i32) {
    gio::prelude::ActionGroupExt::activate_action(appwindow, action, Some(&index.to_variant()));
}

fn slot_menu(appwindow: &RnAppWindow, button: &gtk4::Button, i: usize, len: usize) {
    let popover = gtk4::Popover::new();
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let entry = |label: String, action: &'static str, target: i32, sensitive: bool| {
        let item = gtk4::Button::with_label(&label);
        item.add_css_class("flat");
        item.set_sensitive(sensitive);
        if let Some(child) = item.child().and_downcast::<gtk4::Label>() {
            child.set_halign(gtk4::Align::Start);
        }
        item.connect_clicked(clone!(
            #[weak]
            popover,
            #[weak]
            appwindow,
            move |_| {
                popover.popdown();
                activate(&appwindow, action, target);
            }
        ));
        content.append(&item);
    };
    entry(
        gettext("Save the current pen here"),
        "pen-preset-save-at",
        i as i32,
        true,
    );
    entry(
        gettext("Move left"),
        "pen-preset-move",
        (i as i32) * 100 + i as i32 - 1,
        i > 0,
    );
    entry(
        gettext("Move right"),
        "pen-preset-move",
        (i as i32) * 100 + i as i32 + 1,
        i + 1 < len,
    );
    entry(gettext("Delete"), "pen-preset-remove", i as i32, true);
    popover.set_child(Some(&content));
    // not parented to the pen button itself: the button would take the clicks inside the popover
    let Some(anchor) = button.parent().and_then(|favorites| favorites.parent()) else {
        return;
    };
    if let Some(bounds) = button.compute_bounds(&anchor) {
        popover.set_pointing_to(Some(&gdk::Rectangle::new(
            bounds.x() as i32,
            bounds.y() as i32,
            bounds.width() as i32,
            bounds.height() as i32,
        )));
    }
    popover.set_parent(&anchor);
    popover.connect_closed(|p| p.unparent());
    popover.popup();
}

/// Marks the favorite pen that is the current pen (brand ring).
pub(crate) fn refresh(appwindow: &RnAppWindow) {
    let Some(container) = FAVORITES_BOX.with(|b| b.borrow().as_ref().and_then(|w| w.upgrade()))
    else {
        return;
    };
    let active = appwindow.active_tab_canvas().and_then(|canvas| {
        let engine = canvas.engine_ref();
        (engine.current_pen_style_w_override() == PenStyle::Brush)
            .then(|| with_presets(|p| p.position_of(&engine.current_pen_preset(String::new()))))
            .flatten()
    });
    let mut child = container.first_child();
    let mut i = 0;
    while let Some(widget) = child {
        if widget.has_css_class("onepepe-pen") {
            if active == Some(i) {
                widget.add_css_class("active");
            } else {
                widget.remove_css_class("active");
            }
            i += 1;
        }
        child = widget.next_sibling();
    }
}

pub(crate) fn setup_actions(appwindow: &RnAppWindow) {
    let action_pen_preset = gio::SimpleAction::new("pen-preset", Some(&i32::static_variant_type()));
    action_pen_preset.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, target| {
            let Some(index) = target.and_then(|t| t.get::<i32>()) else {
                return;
            };
            let Some(preset) = with_presets(|p| p.0.get(index as usize).cloned()) else {
                return;
            };
            if let Some(canvas) = appwindow.active_tab_canvas() {
                let widget_flags = canvas.engine_mut().apply_pen_preset(&preset);
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
            appwindow.refresh_ui();
        }
    ));
    appwindow.add_action(&action_pen_preset);

    let action_remove =
        gio::SimpleAction::new("pen-preset-remove", Some(&i32::static_variant_type()));
    action_remove.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, target| {
            let Some(index) = target.and_then(|t| t.get::<i32>()) else {
                return;
            };
            with_presets(|p| p.remove(index as usize));
            save_presets();
            rebuild(&appwindow);
        }
    ));
    appwindow.add_action(&action_remove);

    // target: from * 100 + to
    let action_move = gio::SimpleAction::new("pen-preset-move", Some(&i32::static_variant_type()));
    action_move.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, target| {
            let Some(target) = target.and_then(|t| t.get::<i32>()) else {
                return;
            };
            with_presets(|p| p.move_to((target / 100) as usize, (target % 100) as usize));
            save_presets();
            rebuild(&appwindow);
        }
    ));
    appwindow.add_action(&action_move);

    let action_save_at =
        gio::SimpleAction::new("pen-preset-save-at", Some(&i32::static_variant_type()));
    action_save_at.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, target| {
            let Some(index) = target.and_then(|t| t.get::<i32>()) else {
                return;
            };
            save_current(&appwindow, Some(index as usize));
        }
    ));
    appwindow.add_action(&action_save_at);

    let action_save = gio::SimpleAction::new("pen-preset-save", None);
    action_save.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, _| save_current(&appwindow, None)
    ));
    appwindow.add_action(&action_save);

    for i in 1..=PenPresets::MAX {
        appwindow.app().set_accels_for_action(
            &format!("win.pen-preset({})", i - 1),
            &[&format!("<Ctrl>{i}"), &format!("<Ctrl>KP_{i}")],
        );
    }
}

/// Saves the current pen in a slot, or in the first free one.
fn save_current(appwindow: &RnAppWindow, slot: Option<usize>) {
    let Some(canvas) = appwindow.active_tab_canvas() else {
        return;
    };
    let count = presets_len();
    let name = format!(
        "{} {}",
        gettext("Pen"),
        slot.map(|s| s + 1).unwrap_or(count + 1)
    );
    let preset = canvas.engine_ref().current_pen_preset(name);
    let saved = with_presets(|p| match slot {
        Some(slot) => {
            p.replace(slot, preset);
            true
        }
        None => p.add(preset),
    });
    let message = if saved {
        save_presets();
        rebuild(appwindow);
        gettext("Saved the current pen as favorite")
    } else {
        gettext("The 5 slots are full: right click a pen to save the current one in its place")
    };
    appwindow
        .overlays()
        .dispatch_toast_text(&message, crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT);
}
