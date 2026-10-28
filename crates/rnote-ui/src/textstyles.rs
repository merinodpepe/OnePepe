// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the text styles of the design guide, Title 1 (28/36, 600), Title 2 (22/30, 600) and Normal (16/24, 400),
// shown in the ribbon with a live preview, with Ctrl+Alt+1, Ctrl+Alt+2 and Ctrl+Alt+0. They can be edited; editing
// a style changes all the text of the open documents that uses it (the text with its size and weight).
// Kept in ~/.config/onepepe/ajustes.json.

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{glib, glib::clone, prelude::*};
use rnote_engine::pens::typewriter::scripts::{QuickStyle, TextStyleDef, TextStyles};

const STYLES: [(QuickStyle, &str); 3] = [
    (QuickStyle::Heading1, "heading1"),
    (QuickStyle::Heading2, "heading2"),
    (QuickStyle::Normal, "normal"),
];

fn style_name(style: QuickStyle) -> String {
    match style {
        QuickStyle::Heading1 => gettext("Heading 1"),
        QuickStyle::Heading2 => gettext("Heading 2"),
        QuickStyle::Quote => gettext("Quote"),
        QuickStyle::Code => gettext("Code"),
        QuickStyle::Normal => gettext("Normal"),
    }
}

/// The text styles, as the user left them.
pub(crate) fn current() -> TextStyles {
    crate::prefs::get("estilos-texto")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

fn save(styles: &TextStyles) {
    if let Ok(value) = serde_json::to_value(styles) {
        crate::prefs::set("estilos-texto", value);
    }
}

/// The preview label of a style: the name in (a scaled down) size and weight of the style.
fn preview_markup(style: QuickStyle, def: &TextStyleDef) -> String {
    // the ribbon is 88 px high: the preview keeps the proportions, from 12 px for Normal
    let size = (12.0 * def.size / 16.0).clamp(10.0, 22.0);
    format!(
        "<span size='{}' weight='{}'>{}</span>",
        (size * 1024.0 * 0.75) as i64,
        def.weight,
        glib::markup_escape_text(&style_name(style))
    )
}

thread_local! {
    static PREVIEWS: std::cell::RefCell<Vec<(QuickStyle, glib::WeakRef<gtk4::Label>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn refresh_previews() {
    let styles = current();
    PREVIEWS.with_borrow(|previews| {
        for (style, label) in previews {
            if let (Some(label), Some(def)) = (label.upgrade(), styles.get(*style)) {
                label.set_markup(&preview_markup(*style, &def));
            }
        }
    });
}

/// The gallery of the ribbon: one button per style with its preview, and the menu to edit them.
pub(crate) fn gallery(more_styles: &gtk4::Popover) -> Vec<gtk4::Widget> {
    let styles = current();
    let mut widgets: Vec<gtk4::Widget> = STYLES
        .iter()
        .map(|(style, id)| {
            let label = gtk4::Label::new(None);
            label.set_markup(&preview_markup(
                *style,
                &styles.get(*style).unwrap_or_default(),
            ));
            PREVIEWS.with_borrow_mut(|p| p.push((*style, label.downgrade())));
            let shortcut = match style {
                QuickStyle::Heading1 => "Ctrl+Alt+1",
                QuickStyle::Heading2 => "Ctrl+Alt+2",
                _ => "Ctrl+Alt+0",
            };
            let button = gtk4::Button::builder()
                .child(&label)
                .action_name("win.text-quick-style")
                .action_target(&id.to_variant())
                .tooltip_text(format!("{} ({shortcut})", style_name(*style)))
                .build();
            button.add_css_class("flat");
            button.add_css_class("onepepe-style-preview");
            button.set_focus_on_click(false);
            button.upcast()
        })
        .collect();
    let more = gtk4::MenuButton::builder()
        .icon_name("pan-down-symbolic")
        .tooltip_text(gettext("More styles"))
        .popover(more_styles)
        .valign(gtk4::Align::Center)
        .build();
    more.add_css_class("flat");
    more.set_focus_on_click(false);
    widgets.push(more.upcast());
    widgets
}

/// The popover to edit the styles: size and weight of each one.
pub(crate) fn edit_popover(appwindow: &RnAppWindow) -> gtk4::Popover {
    let grid = gtk4::Grid::builder()
        .row_spacing(6)
        .column_spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let title = gtk4::Label::new(Some(&gettext("Edit text styles")));
    title.add_css_class("heading");
    title.set_halign(gtk4::Align::Start);
    grid.attach(&title, 0, 0, 3, 1);
    let weights = [400u16, 600, 700];
    let weight_names =
        gtk4::StringList::new(&[&gettext("Regular"), &gettext("Semibold"), &gettext("Bold")]);
    let popover = gtk4::Popover::new();
    let mut editors = vec![];
    let styles = current();
    for (row, (style, _)) in STYLES.iter().enumerate() {
        let def = styles.get(*style).unwrap_or_default();
        let name = gtk4::Label::new(Some(&style_name(*style)));
        name.set_halign(gtk4::Align::Start);
        let size = gtk4::SpinButton::with_range(8.0, 96.0, 1.0);
        size.set_value(def.size);
        size.set_tooltip_text(Some(&gettext("Size")));
        let weight = gtk4::DropDown::new(Some(weight_names.clone()), None::<gtk4::Expression>);
        weight.set_selected(weights.iter().position(|w| *w == def.weight).unwrap_or(0) as u32);
        grid.attach(&name, 0, row as i32 + 1, 1, 1);
        grid.attach(&size, 1, row as i32 + 1, 1, 1);
        grid.attach(&weight, 2, row as i32 + 1, 1, 1);
        editors.push((*style, size, weight));
    }
    let note = gtk4::Label::new(Some(&gettext(
        "The text that already uses a style changes with it, in all the open documents.",
    )));
    note.set_wrap(true);
    note.set_max_width_chars(36);
    note.add_css_class("dim-label");
    note.set_halign(gtk4::Align::Start);
    grid.attach(&note, 0, 4, 3, 1);
    let buttons = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    buttons.set_halign(gtk4::Align::End);
    let reset = gtk4::Button::with_label(&gettext("Reset"));
    let apply = gtk4::Button::with_label(&gettext("Apply"));
    apply.add_css_class("suggested-action");
    buttons.append(&reset);
    buttons.append(&apply);
    grid.attach(&buttons, 0, 5, 3, 1);
    let editors = std::rc::Rc::new(editors);
    reset.connect_clicked(clone!(
        #[strong]
        editors,
        move |_| {
            let defaults = TextStyles::default();
            for (style, size, weight) in editors.iter() {
                let def = defaults.get(*style).unwrap_or_default();
                size.set_value(def.size);
                weight.set_selected(weights.iter().position(|w| *w == def.weight).unwrap_or(0) as u32);
            }
        }
    ));
    apply.connect_clicked(clone!(
        #[weak]
        appwindow,
        #[weak]
        popover,
        #[strong]
        editors,
        move |_| {
            let old = current();
            let mut new = old;
            for (style, size, weight) in editors.iter() {
                let mut def = old.get(*style).unwrap_or_default();
                let new_size = size.value();
                // the line height keeps its proportion
                def.line_height = (def.line_height * new_size / def.size).round();
                def.size = new_size;
                def.weight = weights[(weight.selected() as usize).min(weights.len() - 1)];
                new.set(*style, def);
            }
            popover.popdown();
            apply_styles(&appwindow, &old, &new);
        }
    ));
    popover.set_child(Some(&grid));
    popover
}

/// Saves the new styles and changes the text that uses the old ones, in all the open documents.
fn apply_styles(appwindow: &RnAppWindow, old: &TextStyles, new: &TextStyles) {
    save(new);
    refresh_previews();
    let mut changed = 0;
    for (style, _) in STYLES {
        let (Some(old_def), Some(new_def)) = (old.get(style), new.get(style)) else {
            continue;
        };
        if old_def == new_def {
            continue;
        }
        for page in appwindow.tabs_snapshot() {
            let Some(wrapper) = page
                .child()
                .downcast_ref::<crate::RnCanvasWrapper>()
                .cloned()
            else {
                continue;
            };
            let canvas = wrapper.canvas();
            let (widget_flags, n) = canvas.engine_mut().restyle_texts(&old_def, &new_def);
            appwindow.handle_widget_flags(widget_flags, &canvas);
            changed += n;
        }
    }
    let message = if changed > 0 {
        gettext("Styles saved; {} texts changed").replace("{}", &changed.to_string())
    } else {
        gettext("Styles saved")
    };
    appwindow
        .overlays()
        .dispatch_toast_text(&message, crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT);
}

pub(crate) fn setup_accels(appwindow: &RnAppWindow) {
    let app = appwindow.app();
    app.set_accels_for_action(
        "win.text-quick-style::heading1",
        &["<Ctrl><Alt>1", "<Ctrl><Alt>KP_1"],
    );
    app.set_accels_for_action(
        "win.text-quick-style::heading2",
        &["<Ctrl><Alt>2", "<Ctrl><Alt>KP_2"],
    );
    app.set_accels_for_action(
        "win.text-quick-style::normal",
        &["<Ctrl><Alt>0", "<Ctrl><Alt>KP_0"],
    );
}
