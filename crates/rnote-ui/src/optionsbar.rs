// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the floating tool options bar of the design guide. It is the pens sidebar of Rnote (the options of the
// active tool) with, on top, a grip to drag it (up and down, and to the other side of the canvas), a button to
// fold it, a «Customize» menu, and 2-3 tools chosen by the user. Its place and tools are kept in the preferences.

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{glib, glib::clone, prelude::*};
use serde_json::json;
use std::cell::Cell;
use std::rc::Rc;

/// The tools that can be put on the bar: (pen style, icon, name).
fn tools() -> [(&'static str, &'static str, String); 6] {
    [
        ("brush", "pen-brush-symbolic", gettext("Brush")),
        ("shaper", "pen-shaper-symbolic", gettext("Shaper")),
        (
            "typewriter",
            "pen-typewriter-symbolic",
            gettext("Typewriter"),
        ),
        ("eraser", "pen-eraser-symbolic", gettext("Eraser")),
        ("selector", "pen-selector-symbolic", gettext("Selector")),
        ("tools", "pen-tools-symbolic", gettext("Tools")),
    ]
}

const MAX_TOOLS: usize = 3;
const DEFAULT_TOOLS: [&str; 3] = ["brush", "eraser", "selector"];

/// The tools chosen by the user, in order.
fn chosen_tools() -> Vec<String> {
    match crate::prefs::get("barra-herramientas").and_then(|v| v.as_array().cloned()) {
        Some(list) => list
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .filter(|t| tools().iter().any(|(id, ..)| id == t))
            .take(MAX_TOOLS)
            .collect(),
        None => DEFAULT_TOOLS.iter().map(|t| t.to_string()).collect(),
    }
}

/// Which side the bar is anchored to: "left", "right", or "auto" (Rnote's left/right-handed setting).
fn side() -> String {
    crate::prefs::get_string("barra-lado", "auto")
}

pub(crate) fn setup(appwindow: &RnAppWindow) {
    let sidebar_box = appwindow.overlays().sidebar_box();
    let scroller = appwindow.overlays().sidebar_scroller();
    sidebar_box.set_orientation(gtk4::Orientation::Vertical);
    sidebar_box.add_css_class("onepepe-optionsbar");

    // header: grip, fold, customize
    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    header.add_css_class("onepepe-optionsbar-header");
    let grip = gtk4::Image::builder()
        .icon_name("list-drag-handle-symbolic")
        .tooltip_text(gettext("Drag to move the bar"))
        .hexpand(true)
        .build();
    grip.set_cursor_from_name(Some("grab"));
    let fold = gtk4::ToggleButton::builder()
        .icon_name("pan-up-symbolic")
        .tooltip_text(gettext("Fold the bar"))
        .active(crate::prefs::get_bool("barra-plegada", false))
        .build();
    fold.add_css_class("flat");
    fold.set_focus_on_click(false);
    let customize = gtk4::MenuButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text(gettext("Customize"))
        .build();
    customize.add_css_class("flat");
    customize.set_focus_on_click(false);
    header.append(&grip);
    header.append(&fold);
    header.append(&customize);

    let tools_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    tools_box.add_css_class("onepepe-optionsbar-tools");
    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    sidebar_box.prepend(&separator);
    sidebar_box.prepend(&tools_box);
    sidebar_box.prepend(&header);

    let rebuild_tools = clone!(
        #[weak]
        tools_box,
        #[weak]
        separator,
        move || {
            while let Some(child) = tools_box.first_child() {
                tools_box.remove(&child);
            }
            let chosen = chosen_tools();
            for id in &chosen {
                let Some((_, icon, name)) = tools().into_iter().find(|(t, ..)| t == id) else {
                    continue;
                };
                let button = gtk4::ToggleButton::builder()
                    .icon_name(icon)
                    .tooltip_text(name)
                    .action_name("win.pen-style")
                    .action_target(&id.to_variant())
                    .build();
                button.add_css_class("flat");
                button.add_css_class("sidebar_action_button");
                button.set_focus_on_click(false);
                tools_box.append(&button);
            }
            separator.set_visible(!chosen.is_empty());
        }
    );
    let rebuild_tools = Rc::new(rebuild_tools);
    rebuild_tools();

    // fold: only the header stays
    let apply_fold = clone!(
        #[weak]
        scroller,
        #[weak]
        tools_box,
        #[weak]
        separator,
        move |folded: bool| {
            scroller.set_visible(!folded);
            tools_box.set_visible(!folded);
            separator.set_visible(!folded && !chosen_tools().is_empty());
        }
    );
    apply_fold(fold.is_active());
    fold.connect_toggled(move |fold| {
        let folded = fold.is_active();
        fold.set_icon_name(if folded {
            "pan-down-symbolic"
        } else {
            "pan-up-symbolic"
        });
        crate::prefs::set("barra-plegada", folded.into());
        apply_fold(folded);
    });

    customize.set_popover(Some(&customize_popover(appwindow, rebuild_tools)));

    // the side: follow the left/right-handed setting unless the user anchored it
    appwindow.connect_notify_local(Some("righthanded"), |appwindow, _| apply_side(appwindow));
    apply_side(appwindow);
    apply_position(appwindow);

    // drag the grip: up and down, and to the other side of the canvas
    let drag = gtk4::GestureDrag::new();
    let start_margin = Rc::new(Cell::new(0));
    drag.connect_drag_begin(clone!(
        #[weak]
        sidebar_box,
        #[strong]
        start_margin,
        move |_, _, _| {
            let top = sidebar_box
                .compute_point(
                    &sidebar_box.parent().unwrap(),
                    &gtk4::graphene::Point::zero(),
                )
                .map(|p| p.y() as i32)
                .unwrap_or(sidebar_box.margin_top());
            start_margin.set(top);
        }
    ));
    drag.connect_drag_update(clone!(
        #[weak]
        sidebar_box,
        #[strong]
        start_margin,
        move |_, _, dy| {
            let parent_height = sidebar_box.parent().map(|p| p.height()).unwrap_or(0);
            let max = (parent_height - sidebar_box.height() - 8).max(8);
            let top = (start_margin.get() + dy as i32).clamp(8, max);
            sidebar_box.set_valign(gtk4::Align::Start);
            sidebar_box.set_margin_top(top);
        }
    ));
    drag.connect_drag_end(clone!(
        #[weak]
        appwindow,
        #[weak]
        sidebar_box,
        move |gesture, dx, _| {
            crate::prefs::set("barra-y", sidebar_box.margin_top().into());
            // dropped on the other half of the canvas: anchor it there
            let Some(parent) = sidebar_box.parent() else {
                return;
            };
            let Some((x, _)) = gesture.start_point() else {
                return;
            };
            let Some(start) =
                sidebar_box.compute_point(&parent, &gtk4::graphene::Point::new(x as f32, 0.0))
            else {
                return;
            };
            let end_x = start.x() as f64 + dx;
            let now_left = sidebar_box.halign() == gtk4::Align::Start;
            let want_left = end_x < parent.width() as f64 / 2.0;
            if now_left != want_left {
                crate::prefs::set(
                    "barra-lado",
                    if want_left { "left" } else { "right" }.into(),
                );
                // once the drag is over, or the overlay keeps the old place until the next layout
                glib::idle_add_local_once(clone!(
                    #[weak]
                    appwindow,
                    move || {
                        apply_side(&appwindow);
                        appwindow.overlays().sidebar_box().queue_resize();
                    }
                ));
            }
        }
    ));
    grip.add_controller(drag);
}

fn apply_side(appwindow: &RnAppWindow) {
    let left = match side().as_str() {
        "left" => true,
        "right" => false,
        // as Rnote: for right-handed people the bar is on the left, so the hand does not cover it
        _ => appwindow.righthanded(),
    };
    appwindow.overlays().sidebar_box().set_halign(if left {
        gtk4::Align::Start
    } else {
        gtk4::Align::End
    });
}

fn apply_position(appwindow: &RnAppWindow) {
    if let Some(top) = crate::prefs::get("barra-y").and_then(|v| v.as_i64()) {
        let sidebar_box = appwindow.overlays().sidebar_box();
        sidebar_box.set_valign(gtk4::Align::Start);
        sidebar_box.set_margin_top(top as i32);
    }
}

fn customize_popover(
    appwindow: &RnAppWindow,
    rebuild_tools: Rc<impl Fn() + 'static>,
) -> gtk4::Popover {
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    let title = gtk4::Label::new(Some(&gettext("Tools on the bar (up to 3)")));
    title.add_css_class("heading");
    title.set_halign(gtk4::Align::Start);
    content.append(&title);
    let list = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    content.append(&list);

    // the list is rebuilt after every change, so it always shows the order
    let fill: Rc<std::cell::RefCell<Option<Box<dyn Fn()>>>> =
        Rc::new(std::cell::RefCell::new(None));
    {
        let fill_clone = fill.clone();
        let list = list.clone();
        let rebuild_tools = rebuild_tools.clone();
        *fill.borrow_mut() = Some(Box::new(move || {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            let chosen = chosen_tools();
            // the chosen ones first, in their order, then the others
            let mut ordered: Vec<_> = chosen
                .iter()
                .filter_map(|c| tools().into_iter().find(|(t, ..)| t == c))
                .collect();
            ordered.extend(
                tools()
                    .into_iter()
                    .filter(|(t, ..)| !chosen.iter().any(|c| c == t)),
            );
            for (id, icon, name) in ordered {
                let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                let check = gtk4::CheckButton::builder()
                    .active(chosen.iter().any(|c| c == id))
                    .sensitive(chosen.iter().any(|c| c == id) || chosen.len() < MAX_TOOLS)
                    .build();
                row.append(&check);
                row.append(&gtk4::Image::from_icon_name(icon));
                let label = gtk4::Label::new(Some(&name));
                label.set_hexpand(true);
                label.set_halign(gtk4::Align::Start);
                row.append(&label);
                let up = gtk4::Button::from_icon_name("go-up-symbolic");
                up.add_css_class("flat");
                up.set_tooltip_text(Some(&gettext("Move up")));
                up.set_sensitive(
                    chosen.first().is_some_and(|c| c != id) && chosen.iter().any(|c| c == id),
                );
                row.append(&up);
                list.append(&row);

                let update = {
                    let fill = fill_clone.clone();
                    let rebuild_tools = rebuild_tools.clone();
                    move |chosen: Vec<String>| {
                        crate::prefs::set("barra-herramientas", json!(chosen));
                        rebuild_tools();
                        let fill = fill.clone();
                        // rebuild the list once the click is over
                        glib::idle_add_local_once(move || {
                            if let Some(f) = fill.borrow().as_ref() {
                                f();
                            }
                        });
                    }
                };
                let update_check = update.clone();
                check.connect_toggled(move |check| {
                    let mut chosen = chosen_tools();
                    chosen.retain(|c| c != id);
                    if check.is_active() {
                        chosen.push(id.to_string());
                    }
                    update_check(chosen);
                });
                up.connect_clicked(move |_| {
                    let mut chosen = chosen_tools();
                    if let Some(i) = chosen.iter().position(|c| c == id)
                        && i > 0
                    {
                        chosen.swap(i, i - 1);
                    }
                    update(chosen);
                });
            }
        }));
    }
    if let Some(f) = fill.borrow().as_ref() {
        f();
    }

    content.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    let side_title = gtk4::Label::new(Some(&gettext("Place")));
    side_title.add_css_class("heading");
    side_title.set_halign(gtk4::Align::Start);
    content.append(&side_title);
    let auto = gtk4::CheckButton::with_label(&gettext("Automatic (left- or right-handed)"));
    let left = gtk4::CheckButton::with_label(&gettext("Left"));
    let right = gtk4::CheckButton::with_label(&gettext("Right"));
    left.set_group(Some(&auto));
    right.set_group(Some(&auto));
    match side().as_str() {
        "left" => left.set_active(true),
        "right" => right.set_active(true),
        _ => auto.set_active(true),
    }
    for (button, value) in [(&auto, "auto"), (&left, "left"), (&right, "right")] {
        button.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |button| {
                if button.is_active() {
                    crate::prefs::set("barra-lado", value.into());
                    apply_side(&appwindow);
                }
            }
        ));
        content.append(button);
    }
    let reset = gtk4::Button::with_label(&gettext("Center vertically"));
    reset.add_css_class("flat");
    reset.connect_clicked(clone!(
        #[weak]
        appwindow,
        move |_| {
            crate::prefs::set("barra-y", serde_json::Value::Null);
            let sidebar_box = appwindow.overlays().sidebar_box();
            sidebar_box.set_valign(gtk4::Align::Center);
            sidebar_box.set_margin_top(sidebar_box.margin_bottom());
        }
    ));
    content.append(&reset);

    gtk4::Popover::builder().child(&content).build()
}
