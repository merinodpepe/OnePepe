// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the settings of the design guide. The rows of Rnote stay as they are (same behavior); they are put in
// flat tabs with an underline: Canvas and document / Background and lines / Input and pen / Shortcuts /
// Appearance, with the logo on top and a fixed bar at the bottom: changes show at once, «Revert» goes back to how
// it was when the settings were opened (or last applied), «Apply» keeps them.

// Imports
use crate::{RnAppWindow, RnSettingsPanel};
use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{glib, glib::clone};
use rnote_engine::document::DocumentConfig;
use rnote_engine::engine::EngineConfig;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tracing::error;

/// The value of a control of the settings.
#[derive(Debug, Clone, PartialEq)]
enum ControlValue {
    Switch(bool),
    Toggle(bool),
    Spin(f64),
    Combo(u32),
    Color(String),
}

/// What «Revert» brings back.
#[derive(Default)]
struct Baseline {
    engine_config: Option<EngineConfig>,
    document_config: Option<DocumentConfig>,
    window_properties: Vec<(String, glib::Value)>,
    controls: Vec<(gtk4::Widget, ControlValue)>,
    /// To tell whether something changed.
    fingerprint: String,
}

/// The values of all the controls of the settings (many settings of Rnote live only in their row).
fn controls(root: &gtk4::Widget, out: &mut Vec<(gtk4::Widget, ControlValue)>) {
    // the tabs are not a setting
    if root.has_css_class("onepepe-settings-tabs") {
        return;
    }
    let value = if let Some(w) = root.downcast_ref::<adw::SwitchRow>() {
        Some(ControlValue::Switch(w.is_active()))
    } else if let Some(w) = root.downcast_ref::<gtk4::Switch>() {
        Some(ControlValue::Switch(w.is_active()))
    } else if let Some(w) = root.downcast_ref::<adw::SpinRow>() {
        Some(ControlValue::Spin(w.value()))
    } else if let Some(w) = root.downcast_ref::<gtk4::SpinButton>() {
        Some(ControlValue::Spin(w.value()))
    } else if let Some(w) = root.downcast_ref::<adw::ComboRow>() {
        Some(ControlValue::Combo(w.selected()))
    } else if let Some(w) = root.downcast_ref::<gtk4::ToggleButton>() {
        Some(ControlValue::Toggle(w.is_active()))
    } else {
        root.downcast_ref::<gtk4::ColorDialogButton>()
            .map(|w| ControlValue::Color(w.rgba().to_string()))
    };
    if let Some(value) = value {
        // the inner widgets of a control are not settings
        out.push((root.clone(), value));
        return;
    }
    let mut child = root.first_child();
    while let Some(widget) = child {
        controls(&widget, out);
        child = widget.next_sibling();
    }
}

fn restore_control(widget: &gtk4::Widget, value: &ControlValue) {
    match value {
        ControlValue::Switch(on) => {
            if let Some(w) = widget.downcast_ref::<adw::SwitchRow>() {
                w.set_active(*on);
            } else if let Some(w) = widget.downcast_ref::<gtk4::Switch>() {
                w.set_active(*on);
            }
        }
        ControlValue::Toggle(on) => {
            if let Some(w) = widget.downcast_ref::<gtk4::ToggleButton>()
                && w.is_active() != *on
                && *on
            {
                // a button with an action changes the action when clicked
                if w.action_name().is_some() {
                    w.emit_clicked();
                } else {
                    w.set_active(true);
                }
            }
        }
        ControlValue::Spin(v) => {
            if let Some(w) = widget.downcast_ref::<adw::SpinRow>() {
                w.set_value(*v);
            } else if let Some(w) = widget.downcast_ref::<gtk4::SpinButton>() {
                w.set_value(*v);
            }
        }
        ControlValue::Combo(i) => {
            if let Some(w) = widget.downcast_ref::<adw::ComboRow>() {
                w.set_selected(*i);
            }
        }
        ControlValue::Color(c) => {
            if let (Some(w), Ok(rgba)) = (
                widget.downcast_ref::<gtk4::ColorDialogButton>(),
                gtk4::gdk::RGBA::parse(c.as_str()),
            ) {
                w.set_rgba(&rgba);
            }
        }
    }
}

/// The row of a group with this title (the rows of the templates have no id at run time).
fn find_row(root: &gtk4::Widget, title: &str) -> Option<gtk4::Widget> {
    if let Some(row) = root.downcast_ref::<adw::PreferencesRow>()
        && row.title() == title
    {
        return Some(root.clone());
    }
    let mut child = root.first_child();
    while let Some(widget) = child {
        if let Some(found) = find_row(&widget, title) {
            return Some(found);
        }
        child = widget.next_sibling();
    }
    None
}

/// Moves a row of Rnote from its group to another group.
fn move_row(from: &adw::PreferencesGroup, title: &str, to: &adw::PreferencesGroup) {
    match find_row(from.upcast_ref(), title) {
        Some(row) => {
            from.remove(&row);
            to.add(&row);
        }
        None => error!("Settings: no row `{title}`"),
    }
}

fn page(groups: &[gtk4::Widget]) -> gtk4::Box {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
    for g in groups {
        page.append(g);
    }
    page
}

fn window_properties(appwindow: &RnAppWindow) -> Vec<(String, glib::Value)> {
    let own_type = RnAppWindow::static_type();
    appwindow
        .list_properties()
        .iter()
        .filter(|p| p.owner_type() == own_type)
        .filter(|p| p.flags().contains(glib::ParamFlags::READWRITE))
        .map(|p| (p.name().to_string(), appwindow.property_value(p.name())))
        .collect()
}

/// The JSON of a configuration without what changes by itself (the random seed of the brushes, new at each stroke).
fn settings_json<T: serde::Serialize>(value: &T) -> String {
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("seed");
                map.values_mut().for_each(strip);
            }
            serde_json::Value::Array(list) => list.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut value = serde_json::to_value(value).unwrap_or_default();
    strip(&mut value);
    value.to_string()
}

fn fingerprint(appwindow: &RnAppWindow) -> String {
    let mut values = vec![];
    controls(
        appwindow.sidebar().settings_panel().upcast_ref(),
        &mut values,
    );
    let values = values
        .iter()
        .map(|(_, v)| format!("{v:?}"))
        .collect::<Vec<_>>()
        .join(",");
    let engine = settings_json(&*appwindow.engine_config().read());
    let document = appwindow
        .active_tab_canvas()
        .map(|c| settings_json(&c.engine_ref().document.config))
        .unwrap_or_default();
    let window = window_properties(appwindow)
        .iter()
        .map(|(n, v)| format!("{n}={v:?}"))
        .collect::<Vec<_>>()
        .join(";");
    format!("{engine}|{document}|{window}|{values}")
}

fn take_baseline(appwindow: &RnAppWindow) -> Baseline {
    Baseline {
        engine_config: Some(appwindow.engine_config().read().clone()),
        document_config: appwindow
            .active_tab_canvas()
            .map(|c| c.engine_ref().document.config.clone()),
        window_properties: window_properties(appwindow),
        controls: {
            let mut values = vec![];
            controls(
                appwindow.sidebar().settings_panel().upcast_ref(),
                &mut values,
            );
            values
        },
        fingerprint: fingerprint(appwindow),
    }
}

pub(crate) fn setup(panel: &RnSettingsPanel, appwindow: &RnAppWindow) {
    let scroller = panel.settings_scroller();
    let Some(clamp) = scroller
        .child()
        .and_then(|c| {
            c.clone()
                .downcast::<gtk4::Viewport>()
                .ok()
                .and_then(|v| v.child())
                .or(Some(c))
        })
        .and_downcast::<adw::Clamp>()
    else {
        error!("Settings: unexpected layout, kept as in Rnote");
        return;
    };
    let Some(old_box) = clamp.child().and_downcast::<gtk4::Box>() else {
        return;
    };
    // the groups of Rnote, in the order of the .ui file
    let mut groups = vec![];
    let mut child = old_box.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        old_box.remove(&widget);
        groups.push(widget);
    }
    let [general, format, document, buttons, pen_modes] = groups.as_slice() else {
        error!(
            "Settings: {} groups instead of 5, kept as in Rnote",
            groups.len()
        );
        for g in &groups {
            old_box.append(g);
        }
        return;
    };

    // Background and lines: the background rows of the Document group
    let background = adw::PreferencesGroup::builder()
        .title(gettext("Background"))
        .build();
    match document.downcast_ref::<adw::PreferencesGroup>() {
        Some(document) => {
            for title in [
                "Color",
                "Pattern",
                "Pattern Color",
                "Pattern Width",
                "Pattern Height",
                "Invert Color Brightness",
            ] {
                move_row(document, &gettext(title), &background);
            }
        }
        None => error!("Settings: the document group is not a preferences group"),
    }
    let more = adw::ActionRow::builder()
        .title(gettext("Line width, margin line and presets"))
        .subtitle(gettext("In the View tab → Background and lines"))
        .build();
    background.add(&more);

    // Shortcuts: the ones of OnePepe, and the window with all of them
    let shortcuts = adw::PreferencesGroup::builder()
        .title(gettext("Keyboard shortcuts"))
        .build();
    for (title, accel) in [
        (gettext("Favorite pens 1 to 5"), "<Ctrl>1"),
        (
            gettext("Tools: brush, shapes, text, eraser, selection, others"),
            "<Alt>1",
        ),
        (gettext("Heading 1"), "<Ctrl><Alt>1"),
        (gettext("Heading 2"), "<Ctrl><Alt>2"),
        (gettext("Normal"), "<Ctrl><Alt>0"),
        (gettext("Hand (move around the page)"), "<Alt>7"),
        (gettext("Ruler"), "<Ctrl><Alt>r"),
        (gettext("Math mode"), "<Ctrl><Alt>m"),
        (gettext("Show or hide the ribbon"), "<Ctrl>F1"),
    ] {
        let row = adw::ActionRow::builder().title(title).build();
        let keys = gtk4::accelerator_parse(accel)
            .map(|(key, mods)| gtk4::accelerator_get_label(key, mods).to_string())
            .unwrap_or_else(|| accel.to_string());
        let label = gtk4::Label::new(Some(&keys));
        label.add_css_class("dim-label");
        row.add_suffix(&label);
        shortcuts.add(&row);
    }
    let all = adw::ActionRow::builder()
        .title(gettext("All the shortcuts"))
        .activatable(true)
        .action_name("win.keyboard-shortcuts")
        .build();
    all.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    shortcuts.add(&all);

    // Appearance
    let appearance = adw::PreferencesGroup::builder()
        .title(gettext("Appearance"))
        .build();
    let scheme = adw::ActionRow::builder()
        .title(gettext("Theme"))
        .subtitle(gettext("All the window changes together"))
        .build();
    let scheme_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    scheme_box.add_css_class("linked");
    scheme_box.set_valign(gtk4::Align::Center);
    for (label, target) in [
        (gettext("System"), "default"),
        (gettext("Light"), "force-light"),
        (gettext("Dark"), "force-dark"),
    ] {
        let button = gtk4::ToggleButton::builder()
            .label(label)
            .action_name("app.color-scheme")
            .action_target(&target.to_variant())
            .build();
        scheme_box.append(&button);
    }
    scheme.add_suffix(&scheme_box);
    appearance.add(&scheme);
    let switch_row = |title: String, subtitle: String, action: &str| {
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();
        let switch = gtk4::Switch::builder()
            .valign(gtk4::Align::Center)
            .action_name(action)
            .build();
        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));
        row
    };
    appearance.add(&switch_row(
        gettext("Dark canvas"),
        gettext("In the dark theme, the page is shown dark (only on screen)"),
        "win.dark-canvas",
    ));
    appearance.add(&switch_row(
        gettext("Right-handed"),
        gettext("Off: the panels and the tool bar change sides, for left-handed people"),
        "win.righthanded",
    ));
    appearance.add(&switch_row(
        gettext("Ribbon"),
        gettext("Show the ribbon (Ctrl+F1)"),
        "win.ribbon",
    ));

    let pages = [
        (
            gettext("Canvas and document"),
            page(&[general.clone(), format.clone(), document.clone()]),
        ),
        (
            gettext("Background and lines"),
            page(&[background.upcast()]),
        ),
        (
            gettext("Input and pen"),
            page(&[buttons.clone(), pen_modes.clone()]),
        ),
        (gettext("Shortcuts"), page(&[shortcuts.upcast()])),
        (gettext("Appearance"), page(&[appearance.upcast()])),
    ];
    let stack = gtk4::Stack::builder()
        .transition_type(gtk4::StackTransitionType::Crossfade)
        .transition_duration(150)
        .vhomogeneous(false)
        .build();
    for (i, (title, page)) in pages.iter().enumerate() {
        stack.add_titled(page, Some(&format!("page{i}")), title);
    }
    old_box.append(&stack);
    panel.add_css_class("onepepe-settings");

    // header: the logo and the tabs
    let header = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    header.add_css_class("onepepe-settings-header");
    let title_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    title_box.append(
        &gtk4::Image::builder()
            .icon_name("onepepe-logo-symbolic")
            .pixel_size(32)
            .build(),
    );
    let title = gtk4::Label::new(Some(&gettext("OnePepe settings")));
    title.add_css_class("title-3");
    title_box.append(&title);
    header.append(&title_box);
    // flat tabs, each as wide as its name
    let tabs = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    tabs.add_css_class("onepepe-settings-tabs");
    let mut first: Option<gtk4::ToggleButton> = None;
    for (i, (title, _)) in pages.iter().enumerate() {
        let button = gtk4::ToggleButton::builder().label(title.as_str()).build();
        if let Some(first) = &first {
            button.set_group(Some(first));
        } else {
            button.set_active(true);
            first = Some(button.clone());
        }
        button.connect_toggled(clone!(
            #[weak]
            stack,
            move |button| {
                if button.is_active() {
                    stack.set_visible_child_name(&format!("page{i}"));
                }
            }
        ));
        tabs.append(&button);
    }
    let tabs_scroller = gtk4::ScrolledWindow::builder()
        .child(&tabs)
        .vscrollbar_policy(gtk4::PolicyType::Never)
        .hscrollbar_policy(gtk4::PolicyType::External)
        .build();
    header.append(&tabs_scroller);

    // the fixed bar at the bottom
    let bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    bar.add_css_class("onepepe-settings-bar");
    let status = gtk4::Label::new(None);
    status.set_hexpand(true);
    status.set_halign(gtk4::Align::Start);
    status.add_css_class("dim-label");
    let revert = gtk4::Button::with_label(&gettext("Revert"));
    let apply = gtk4::Button::with_label(&gettext("Apply"));
    apply.add_css_class("suggested-action");
    bar.append(&status);
    bar.append(&revert);
    bar.append(&apply);

    // the scroller of Rnote goes between the header and the bar
    let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    scroller.unparent();
    outer.append(&header);
    outer.append(&scroller);
    outer.append(&bar);
    outer.set_parent(panel);

    let baseline = Rc::new(RefCell::new(Baseline::default()));
    let update = clone!(
        #[weak]
        appwindow,
        #[weak]
        status,
        #[weak]
        revert,
        #[weak]
        apply,
        #[strong]
        baseline,
        move || {
            let changed = baseline.borrow().fingerprint != fingerprint(&appwindow);
            status.set_label(&if changed {
                gettext("Changes not applied")
            } else {
                String::new()
            });
            revert.set_sensitive(changed);
            apply.set_sensitive(changed);
        }
    );
    let update = Rc::new(update);
    // the baseline is taken when the settings are shown
    panel.connect_map(clone!(
        #[weak]
        appwindow,
        #[strong]
        baseline,
        #[strong]
        update,
        move |_| {
            baseline.replace(take_baseline(&appwindow));
            update();
        }
    ));
    apply.connect_clicked(clone!(
        #[weak]
        appwindow,
        #[strong]
        baseline,
        #[strong]
        update,
        move |_| {
            baseline.replace(take_baseline(&appwindow));
            appwindow.overlays().dispatch_toast_text(
                &gettext("Settings applied"),
                crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
            );
            update();
        }
    ));
    revert.connect_clicked(clone!(
        #[weak]
        appwindow,
        #[strong]
        baseline,
        #[strong]
        update,
        move |_| {
            {
                let base = baseline.borrow();
                // the rows first (they apply their value), then the configurations, which have the last word
                for (widget, value) in &base.controls {
                    restore_control(widget, value);
                }
                if let Some(config) = &base.engine_config {
                    *appwindow.engine_config().write() = config.clone();
                }
                for (name, value) in &base.window_properties {
                    appwindow.set_property_from_value(name, value);
                }
                if let (Some(config), Some(canvas)) =
                    (&base.document_config, appwindow.active_tab_canvas())
                {
                    let widget_flags = {
                        let mut engine = canvas.engine_mut();
                        engine.document.config = config.clone();
                        engine.doc_resize_autoexpand() | engine.background_rendering_regenerate()
                    };
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            }
            appwindow.refresh_ui();
            update();
        }
    ));
    // there is no signal for every setting: check twice a second while the settings are shown
    glib::timeout_add_local(
        Duration::from_millis(500),
        clone!(
            #[weak]
            panel,
            #[strong]
            update,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                if panel.is_mapped() {
                    update();
                }
                glib::ControlFlow::Continue
            }
        ),
    );
}
