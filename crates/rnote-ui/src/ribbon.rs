// OnePepe: the ribbon at the top of the window, like the ribbon of OneNote, with the tabs
// File, Home, Insert, Draw and View. It is built in code on top of the window actions (`win.*`).

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, prelude::*, subclass::prelude::*};
use rnote_engine::engine::onepepe_insert::{FitMode, PageSize, PageTemplate};
use rnote_engine::pens::penpresets::{PenPreset, PenPresets};
use rnote_engine::pens::pensconfig::toolsconfig::ToolStyle;
use rnote_engine::pens::typewriter::scripts::{DateTimeStamp, ListKind, QuickStyle};
use rnote_engine::strokes::textstroke::TextAttribute;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, error};

thread_local! {
    /// The favorite pens, loaded from the settings file.
    static PEN_PRESETS: RefCell<Option<PenPresets>> = const { RefCell::new(None) };
    /// The box of the ribbon that shows the favorite pens.
    static FAVORITES_BOX: RefCell<Option<glib::WeakRef<gtk4::Box>>> = const { RefCell::new(None) };
}

/// Builds the ribbon, puts it above the tabs and registers its actions.
pub(crate) fn setup(appwindow: &RnAppWindow) {
    setup_actions(appwindow);

    let stack = gtk4::Stack::builder()
        .transition_type(gtk4::StackTransitionType::Crossfade)
        .hhomogeneous(false)
        .vhomogeneous(true)
        .build();
    stack.add_titled(&file_page(appwindow), Some("file"), &gettext("File"));
    stack.add_titled(&home_page(appwindow), Some("home"), &gettext("Home"));
    stack.add_titled(&insert_page(appwindow), Some("insert"), &gettext("Insert"));
    stack.add_titled(&draw_page(appwindow), Some("draw"), &gettext("Draw"));
    stack.add_titled(&view_page(appwindow), Some("view"), &gettext("View"));
    stack.set_visible_child_name("home");

    let switcher = gtk4::StackSwitcher::builder().stack(&stack).build();
    let hide_button = gtk4::ToggleButton::builder()
        .icon_name("pan-up-symbolic")
        .tooltip_text(gettext("Show or hide the ribbon (Ctrl+F1)"))
        .action_name("win.ribbon")
        .valign(gtk4::Align::Center)
        .build();
    hide_button.add_css_class("flat");
    hide_button.set_focus_on_click(false);
    // the tab buttons must not take the keyboard focus away from the text either
    let mut tab_button = switcher.first_child();
    while let Some(child) = tab_button {
        child.set_focus_on_click(false);
        tab_button = child.next_sibling();
    }
    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    header.set_margin_start(6);
    header.set_margin_end(6);
    header.set_margin_top(3);
    header.append(&switcher);
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);
    header.append(&hide_button);

    let revealer = gtk4::Revealer::builder()
        .child(&stack)
        .reveal_child(true)
        .transition_type(gtk4::RevealerTransitionType::SlideDown)
        .build();
    if let Some(action) = appwindow
        .lookup_action("ribbon")
        .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
    {
        action.connect_state_notify(clone!(
            #[weak]
            revealer,
            move |action| {
                let shown = action.state().and_then(|s| s.get::<bool>()).unwrap_or(true);
                revealer.set_reveal_child(shown);
            }
        ));
    }

    let ribbon = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    ribbon.add_css_class("toolbar");
    // never take vertical space from the canvas
    ribbon.set_vexpand(false);
    ribbon.append(&header);
    ribbon.append(&revealer);
    ribbon.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    match appwindow
        .imp()
        .tabbar
        .get()
        .parent()
        .and_downcast::<gtk4::Box>()
    {
        Some(content_box) => content_box.prepend(&ribbon),
        None => error!("Could not place the ribbon: the tab bar has no box as parent"),
    }
}

/// Keeps the toggles of the ribbon in sync with the settings (called from the window refresh).
pub(crate) fn refresh(appwindow: &RnAppWindow) {
    let shape_recognition = appwindow
        .engine_config()
        .read()
        .pens_config
        .brush_config
        .shape_recognition;
    if let Some(action) = appwindow
        .lookup_action("shape-recognition")
        .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
    {
        action.set_state(&shape_recognition.to_variant());
    }
}

// ----- building blocks

/// A big button with an icon on top and a label below, like the buttons of the OneNote ribbon.
fn big_button(
    icon: &str,
    label: &str,
    action: &str,
    target: Option<glib::Variant>,
) -> gtk4::Button {
    let button = gtk4::Button::builder()
        .child(&icon_label(icon, label))
        .action_name(action)
        .tooltip_text(label)
        .build();
    if let Some(target) = target {
        button.set_action_target_value(Some(&target));
    }
    button.add_css_class("flat");
    button.set_focus_on_click(false);
    button
}

/// A small button with an icon only.
fn small_button(
    icon: &str,
    tooltip: &str,
    action: &str,
    target: Option<glib::Variant>,
) -> gtk4::Button {
    let button = gtk4::Button::builder()
        .icon_name(available_icon(icon))
        .action_name(action)
        .tooltip_text(tooltip)
        .build();
    if let Some(target) = target {
        button.set_action_target_value(Some(&target));
    }
    button.add_css_class("flat");
    button.set_focus_on_click(false);
    button
}

/// A small button with a text label.
fn text_button(
    label: &str,
    tooltip: &str,
    action: &str,
    target: Option<glib::Variant>,
) -> gtk4::Button {
    let button = gtk4::Button::builder()
        .label(label)
        .action_name(action)
        .tooltip_text(tooltip)
        .build();
    if let Some(target) = target {
        button.set_action_target_value(Some(&target));
    }
    button.add_css_class("flat");
    button.set_focus_on_click(false);
    button
}

/// A big toggle button (for stateful actions: the pen style, the ruler...).
fn big_toggle(
    icon: &str,
    label: &str,
    action: &str,
    target: Option<glib::Variant>,
) -> gtk4::ToggleButton {
    let button = gtk4::ToggleButton::builder()
        .child(&icon_label(icon, label))
        .action_name(action)
        .tooltip_text(label)
        .build();
    if let Some(target) = target {
        button.set_action_target_value(Some(&target));
    }
    button.add_css_class("flat");
    button.set_focus_on_click(false);
    button
}

/// A big button that opens a popover.
fn big_menu(icon: &str, label: &str, popover: &gtk4::Popover) -> gtk4::MenuButton {
    let button = gtk4::MenuButton::builder()
        .child(&icon_label(icon, label))
        .popover(popover)
        .tooltip_text(label)
        .build();
    button.add_css_class("flat");
    button.set_focus_on_click(false);
    button
}

fn icon_label(icon: &str, label: &str) -> gtk4::Box {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    let image = gtk4::Image::from_icon_name(&available_icon(icon));
    image.set_pixel_size(24);
    vbox.append(&image);
    let label = gtk4::Label::new(Some(label));
    label.add_css_class("caption");
    vbox.append(&label);
    vbox
}

/// The icon, or a plain dot when the icon theme of the system does not have it.
fn available_icon(icon: &str) -> String {
    let found = gtk4::gdk::Display::default()
        .map(|display| gtk4::IconTheme::for_display(&display).has_icon(icon))
        .unwrap_or(true);
    if found {
        icon.to_string()
    } else {
        "dot-symbolic".to_string()
    }
}

/// A group of the ribbon: its buttons and its name below.
fn group(name: &str, children: &[gtk4::Widget]) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    row.set_valign(gtk4::Align::Center);
    for child in children {
        row.append(child);
    }
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    vbox.append(&row);
    let label = gtk4::Label::new(Some(name));
    label.add_css_class("caption");
    label.add_css_class("dim-label");
    vbox.append(&label);
    vbox.set_margin_start(4);
    vbox.set_margin_end(4);
    vbox
}

/// Small buttons stacked in rows inside a group.
fn rows(rows: &[&[gtk4::Widget]]) -> gtk4::Widget {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    vbox.set_valign(gtk4::Align::Center);
    for row in rows {
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        for child in *row {
            hbox.append(child);
        }
        vbox.append(&hbox);
    }
    vbox.upcast()
}

/// A page of the ribbon: the groups side by side, scrolling sideways in narrow windows.
fn page(groups: &[gtk4::Box]) -> gtk4::ScrolledWindow {
    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    hbox.set_margin_top(4);
    hbox.set_margin_bottom(4);
    hbox.set_margin_start(4);
    hbox.set_margin_end(4);
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            hbox.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
        }
        hbox.append(group);
    }
    gtk4::ScrolledWindow::builder()
        .child(&hbox)
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Never)
        .propagate_natural_height(true)
        .build()
}

/// A popover with a vertical list of buttons for one action with string targets.
fn list_popover(action: &str, entries: &[(String, &str)]) -> gtk4::Popover {
    let popover = gtk4::Popover::new();
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    for (label, target) in entries {
        let button = gtk4::Button::builder()
            .label(label)
            .action_name(action)
            .action_target(&target.to_variant())
            .build();
        button.add_css_class("flat");
        button.set_focus_on_click(false);
        button.connect_clicked(clone!(
            #[weak]
            popover,
            move |_| popover.popdown()
        ));
        vbox.append(&button);
    }
    popover.set_child(Some(&vbox));
    popover
}

fn s(target: &str) -> Option<glib::Variant> {
    Some(target.to_variant())
}

// ----- pages

fn file_page(_appwindow: &RnAppWindow) -> gtk4::ScrolledWindow {
    let export_popover = list_popover(
        "win.export-fitted",
        &[
            (gettext("PDF on one page"), "one-page"),
            (gettext("PDF to the page width"), "page-width"),
        ],
    );
    page(&[
        group(
            &gettext("Document"),
            &[
                big_button("doc-create-symbolic", &gettext("New"), "win.new-doc", None).upcast(),
                big_button(
                    "workspacelistentryicon-folder-symbolic",
                    &gettext("Open"),
                    "win.open-doc",
                    None,
                )
                .upcast(),
                big_button("save-symbolic", &gettext("Save"), "win.save-doc", None).upcast(),
                big_button(
                    "save-symbolic",
                    &gettext("Save As"),
                    "win.save-doc-as",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Export and print"),
            &[
                big_button(
                    "workspacelistentryicon-document-symbolic",
                    &gettext("Export"),
                    "win.export-doc",
                    None,
                )
                .upcast(),
                big_menu(
                    "zoom-fit-width-symbolic",
                    &gettext("Export fitted"),
                    &export_popover,
                )
                .upcast(),
                big_button(
                    "workspacelistentryicon-library-symbolic",
                    &gettext("Export section"),
                    "win.export-section",
                    None,
                )
                .upcast(),
                big_button(
                    "document-print-symbolic",
                    &gettext("Print"),
                    "win.print-doc",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Backups"),
            &[big_button(
                "workspacelistentryicon-clock-symbolic",
                &gettext("Backups"),
                "win.open-backups",
                None,
            )
            .upcast()],
        ),
        group(
            &gettext("App"),
            &[
                big_button(
                    "settings-symbolic",
                    &gettext("Settings"),
                    "win.open-settings",
                    None,
                )
                .upcast(),
                big_button(
                    "keyboard-ctrl-space-shortcut-symbolic",
                    &gettext("Shortcuts"),
                    "win.keyboard-shortcuts",
                    None,
                )
                .upcast(),
                big_button(
                    "workspacelistentryicon-lightbulb-symbolic",
                    &gettext("About"),
                    "win.about",
                    None,
                )
                .upcast(),
            ],
        ),
    ])
}

fn home_page(appwindow: &RnAppWindow) -> gtk4::ScrolledWindow {
    let styles_popover = list_popover(
        "win.text-quick-style",
        &[
            (gettext("Heading 1"), "heading1"),
            (gettext("Heading 2"), "heading2"),
            (gettext("Quote"), "quote"),
            (gettext("Code"), "code"),
            (gettext("Normal"), "normal"),
        ],
    );
    let datetime_popover = list_popover(
        "win.insert-datetime",
        &[
            (gettext("Date"), "date"),
            (gettext("Time"), "time"),
            (gettext("Date and Time"), "both"),
        ],
    );
    let _ = appwindow;
    page(&[
        group(
            &gettext("Clipboard"),
            &[
                big_button(
                    "clipboard-paste-symbolic",
                    &gettext("Paste"),
                    "win.clipboard-paste",
                    None,
                )
                .upcast(),
                rows(&[
                    &[small_button(
                        "edit-cut-symbolic",
                        &gettext("Cut"),
                        "win.clipboard-cut",
                        None,
                    )
                    .upcast()],
                    &[small_button(
                        "clipboard-copy-symbolic",
                        &gettext("Copy"),
                        "win.clipboard-copy",
                        None,
                    )
                    .upcast()],
                ]),
            ],
        ),
        group(
            &gettext("Undo"),
            &[
                big_button("edit-undo-symbolic", &gettext("Undo"), "win.undo", None).upcast(),
                big_button("edit-redo-symbolic", &gettext("Redo"), "win.redo", None).upcast(),
            ],
        ),
        group(
            &gettext("Text"),
            &[rows(&[
                &[
                    small_button(
                        "text-bold-symbolic",
                        &gettext("Bold"),
                        "win.text-bold",
                        None,
                    )
                    .upcast(),
                    small_button(
                        "text-italic-symbolic",
                        &gettext("Italic"),
                        "win.text-italic",
                        None,
                    )
                    .upcast(),
                    small_button(
                        "text-underline-symbolic",
                        &gettext("Underline"),
                        "win.text-underline",
                        None,
                    )
                    .upcast(),
                    small_button(
                        "text-strikethrough-symbolic",
                        &gettext("Strikethrough"),
                        "win.text-strikethrough",
                        None,
                    )
                    .upcast(),
                ],
                &[
                    text_button("x²", &gettext("Superscript"), "win.text-script", s("sup"))
                        .upcast(),
                    text_button("x₂", &gettext("Subscript"), "win.text-script", s("sub")).upcast(),
                    text_button("•", &gettext("Bullet list"), "win.text-list", s("bullet"))
                        .upcast(),
                    text_button(
                        "1.",
                        &gettext("Numbered list"),
                        "win.text-list",
                        s("numbered"),
                    )
                    .upcast(),
                    text_button("☐", &gettext("Checkbox"), "win.text-list", s("checkbox")).upcast(),
                ],
            ])],
        ),
        group(
            &gettext("Styles"),
            &[
                big_button(
                    "pen-typewriter-fontchooser-symbolic",
                    &gettext("Heading 1"),
                    "win.text-quick-style",
                    s("heading1"),
                )
                .upcast(),
                big_button(
                    "pen-typewriter-fontchooser-symbolic",
                    &gettext("Heading 2"),
                    "win.text-quick-style",
                    s("heading2"),
                )
                .upcast(),
                big_menu(
                    "misc-menu-symbolic",
                    &gettext("More styles"),
                    &styles_popover,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Selection"),
            &[rows(&[
                &[
                    small_button(
                        "selection-select-all-symbolic",
                        &gettext("Select all"),
                        "win.selection-select-all",
                        None,
                    )
                    .upcast(),
                    small_button(
                        "selection-duplicate-symbolic",
                        &gettext("Duplicate"),
                        "win.selection-duplicate",
                        None,
                    )
                    .upcast(),
                ],
                &[
                    small_button(
                        "selection-deselect-all-symbolic",
                        &gettext("Deselect all"),
                        "win.selection-deselect-all",
                        None,
                    )
                    .upcast(),
                    small_button(
                        "selection-trash-symbolic",
                        &gettext("Delete selection"),
                        "win.selection-trash",
                        None,
                    )
                    .upcast(),
                ],
            ])],
        ),
        group(
            &gettext("Date"),
            &[big_menu(
                "workspacelistentryicon-calendar-symbolic",
                &gettext("Date and Time"),
                &datetime_popover,
            )
            .upcast()],
        ),
    ])
}

fn insert_page(appwindow: &RnAppWindow) -> gtk4::ScrolledWindow {
    let templates_popover = list_popover(
        "win.insert-template",
        &[
            (gettext("Title and date"), "title-date"),
            (gettext("Class notes"), "class-notes"),
            (gettext("Cornell notes"), "cornell"),
            (gettext("Problem sheet"), "problems"),
        ],
    );
    let datetime_popover = list_popover(
        "win.insert-datetime",
        &[
            (gettext("Date"), "date"),
            (gettext("Time"), "time"),
            (gettext("Date and Time"), "both"),
        ],
    );
    let symbols_popover = crate::penssidebar::typewriterpage::symbol_catalog_popover(appwindow);
    page(&[
        group(
            &gettext("Space"),
            &[big_button(
                "pen-tools-verticalspacetool-symbolic",
                &gettext("Insert space"),
                "win.tool-style",
                s("verticalspace"),
            )
            .upcast()],
        ),
        group(
            &gettext("Tables"),
            &[big_menu(
                "shapebuilder-grid-symbolic",
                &gettext("Table"),
                &table_popover(),
            )
            .upcast()],
        ),
        group(
            &gettext("Files"),
            &[
                big_button(
                    "workspacelistentryicon-crop-symbolic",
                    &gettext("Picture"),
                    "win.import-file",
                    None,
                )
                .upcast(),
                big_button(
                    "workspacelistentryicon-document-symbolic",
                    &gettext("PDF printout"),
                    "win.import-file",
                    None,
                )
                .upcast(),
                big_button(
                    "workspacelistentryicon-camera-symbolic",
                    &gettext("Screen clipping"),
                    "win.insert-screenshot",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Content"),
            &[
                big_toggle(
                    "pen-typewriter-symbolic",
                    &gettext("Text box"),
                    "win.pen-style",
                    s("typewriter"),
                )
                .upcast(),
                big_toggle(
                    "pen-shaper-symbolic",
                    &gettext("Shapes"),
                    "win.pen-style",
                    s("shaper"),
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Time stamp"),
            &[big_menu(
                "workspacelistentryicon-calendar-symbolic",
                &gettext("Date and Time"),
                &datetime_popover,
            )
            .upcast()],
        ),
        group(
            &gettext("Pages"),
            &[
                big_menu(
                    "doc-create-symbolic",
                    &gettext("Page templates"),
                    &templates_popover,
                )
                .upcast(),
                big_button(
                    "add-page-symbolic",
                    &gettext("Add page"),
                    "win.add-page-to-doc",
                    None,
                )
                .upcast(),
                big_button(
                    "workspacelistentryicon-scratchpad-symbolic",
                    &gettext("Quick note"),
                    "win.quick-note",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Symbols"),
            &[big_menu(
                "workspacelistentryicon-math-symbolic",
                &gettext("Symbol"),
                &symbols_popover,
            )
            .upcast()],
        ),
    ])
}

fn draw_page(appwindow: &RnAppWindow) -> gtk4::ScrolledWindow {
    let favorites = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    favorites.set_valign(gtk4::Align::Center);
    FAVORITES_BOX.with(|b| *b.borrow_mut() = Some(favorites.downgrade()));
    rebuild_favorites(appwindow);

    page(&[
        group(
            &gettext("Tools"),
            &[
                big_toggle(
                    "pen-brush-symbolic",
                    &gettext("Pen"),
                    "win.pen-style",
                    s("brush"),
                )
                .upcast(),
                big_toggle(
                    "pen-eraser-symbolic",
                    &gettext("Eraser"),
                    "win.pen-style",
                    s("eraser"),
                )
                .upcast(),
                big_toggle(
                    "pen-selector-symbolic",
                    &gettext("Lasso"),
                    "win.pen-style",
                    s("selector"),
                )
                .upcast(),
                big_toggle(
                    "pen-shaper-symbolic",
                    &gettext("Shapes"),
                    "win.pen-style",
                    s("shaper"),
                )
                .upcast(),
                big_toggle(
                    "pen-typewriter-symbolic",
                    &gettext("Text"),
                    "win.pen-style",
                    s("typewriter"),
                )
                .upcast(),
                big_button(
                    "pen-tools-laser-symbolic",
                    &gettext("Laser"),
                    "win.tool-style",
                    s("laser"),
                )
                .upcast(),
            ],
        ),
        group(&gettext("Favorite pens"), &[favorites.upcast()]),
        group(
            &gettext("Help"),
            &[
                big_toggle(
                    "onepepe-ruler-symbolic",
                    &gettext("Ruler"),
                    "win.ruler",
                    None,
                )
                .upcast(),
                big_toggle(
                    "onepepe-ink-to-shape-symbolic",
                    &gettext("Ink to Shape"),
                    "win.shape-recognition",
                    None,
                )
                .upcast(),
                big_toggle(
                    "onepepe-math-symbolic",
                    &gettext("Math Mode"),
                    "win.math-mode",
                    None,
                )
                .upcast(),
            ],
        ),
    ])
}

fn view_page(_appwindow: &RnAppWindow) -> gtk4::ScrolledWindow {
    let size_popover = page_size_popover();
    page(&[
        group(
            &gettext("Zoom"),
            &[
                big_button("zoom-in-symbolic", &gettext("Zoom in"), "win.zoom-in", None).upcast(),
                big_button(
                    "zoom-out-symbolic",
                    &gettext("Zoom out"),
                    "win.zoom-out",
                    None,
                )
                .upcast(),
                big_button(
                    "zoom-real-width-symbolic",
                    &gettext("100 %"),
                    "win.zoom-reset",
                    None,
                )
                .upcast(),
                big_button(
                    "zoom-fit-width-symbolic",
                    &gettext("Page width"),
                    "win.zoom-fit-width",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Page"),
            &[
                big_menu(
                    "workspacelistentryicon-document-symbolic",
                    &gettext("Page size"),
                    &size_popover,
                )
                .upcast(),
                big_button(
                    "resize-to-fit-content-symbolic",
                    &gettext("Fit to content"),
                    "win.resize-to-fit-content",
                    None,
                )
                .upcast(),
                big_button(
                    "return-origin-page-symbolic",
                    &gettext("Back to the start"),
                    "win.return-origin-page",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Window"),
            &[
                big_toggle(
                    "focus-mode-symbolic",
                    &gettext("Focus mode"),
                    "win.focus-mode",
                    None,
                )
                .upcast(),
                big_toggle(
                    "appwindow-fullscreen-symbolic",
                    &gettext("Full screen"),
                    "win.fullscreen",
                    None,
                )
                .upcast(),
                big_toggle("pan-up-symbolic", &gettext("Ribbon"), "win.ribbon", None).upcast(),
            ],
        ),
    ])
}

/// The popover of the page sizes: one row per size, portrait and landscape side by side.
fn page_size_popover() -> gtk4::Popover {
    let popover = gtk4::Popover::new();
    let grid = gtk4::Grid::builder()
        .row_spacing(2)
        .column_spacing(6)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    for (row, (name, id)) in [
        ("A5", "a5"),
        ("A4", "a4"),
        ("A3", "a3"),
        ("A2", "a2"),
        ("A1", "a1"),
        ("A0", "a0"),
        ("Letter", "letter"),
    ]
    .into_iter()
    .enumerate()
    {
        let label = gtk4::Label::new(Some(&gettext(name)));
        label.add_css_class("heading");
        label.set_halign(gtk4::Align::Start);
        grid.attach(&label, 0, row as i32, 1, 1);
        for (col, (text, target)) in [
            (gettext("portrait"), id.to_string()),
            (gettext("landscape"), format!("{id}-landscape")),
        ]
        .into_iter()
        .enumerate()
        {
            let button = gtk4::Button::builder()
                .label(text)
                .action_name("win.page-size")
                .action_target(&target.to_variant())
                .build();
            button.add_css_class("flat");
            button.connect_clicked(clone!(
                #[weak]
                popover,
                move |_| popover.popdown()
            ));
            grid.attach(&button, col as i32 + 1, row as i32, 1, 1);
        }
    }
    popover.set_child(Some(&grid));
    popover
}

/// The popover to choose the number of rows and columns of a table.
fn table_popover() -> gtk4::Popover {
    let popover = gtk4::Popover::new();
    let grid = gtk4::Grid::builder()
        .row_spacing(6)
        .column_spacing(6)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let rows_spin = gtk4::SpinButton::with_range(1.0, 30.0, 1.0);
    rows_spin.set_value(3.0);
    let cols_spin = gtk4::SpinButton::with_range(1.0, 15.0, 1.0);
    cols_spin.set_value(3.0);
    grid.attach(&gtk4::Label::new(Some(&gettext("Rows"))), 0, 0, 1, 1);
    grid.attach(&rows_spin, 1, 0, 1, 1);
    grid.attach(&gtk4::Label::new(Some(&gettext("Columns"))), 0, 1, 1, 1);
    grid.attach(&cols_spin, 1, 1, 1, 1);
    let insert = gtk4::Button::with_label(&gettext("Insert table"));
    insert.add_css_class("suggested-action");
    insert.connect_clicked(clone!(
        #[weak]
        popover,
        #[weak]
        rows_spin,
        #[weak]
        cols_spin,
        move |button| {
            let target = format!("{}x{}", rows_spin.value_as_int(), cols_spin.value_as_int());
            let _ = button.activate_action("win.insert-table", Some(&target.to_variant()));
            popover.popdown();
        }
    ));
    grid.attach(&insert, 0, 2, 2, 1);
    popover.set_child(Some(&grid));
    popover
}

// ----- favorite pens

fn presets_path() -> std::path::PathBuf {
    glib::user_config_dir()
        .join("onepepe")
        .join("boligrafos.json")
}

fn with_presets<R>(f: impl FnOnce(&mut PenPresets) -> R) -> R {
    PEN_PRESETS.with(|p| {
        let mut p = p.borrow_mut();
        let presets = p.get_or_insert_with(|| match std::fs::read_to_string(presets_path()) {
            Ok(json) => PenPresets::from_json(&json),
            Err(_) => PenPresets::default(),
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

/// A round swatch with the color and (roughly) the width of the pen. Highlighters are drawn as a bar.
fn swatch(preset: &PenPreset) -> gtk4::DrawingArea {
    let area = gtk4::DrawingArea::builder()
        .content_width(26)
        .content_height(26)
        .build();
    let color = preset.color;
    let marker = matches!(
        preset.style,
        rnote_engine::pens::pensconfig::brushconfig::BrushStyle::Marker
    );
    let radius = (3.0 + preset.width.sqrt() * 2.0).min(12.0);
    area.set_draw_func(move |_, cx, w, h| {
        cx.set_source_rgba(color.r, color.g, color.b, color.a);
        if marker {
            cx.rectangle(2.0, h as f64 * 0.35, w as f64 - 4.0, h as f64 * 0.3);
        } else {
            cx.arc(
                w as f64 / 2.0,
                h as f64 / 2.0,
                radius,
                0.0,
                std::f64::consts::TAU,
            );
        }
        let _ = cx.fill();
    });
    area
}

fn rebuild_favorites(appwindow: &RnAppWindow) {
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
            .tooltip_text(format!(
                "{} ({})",
                preset.name,
                gettext("right click to remove")
            ))
            .action_name("win.pen-preset")
            .action_target(&(i as i32).to_variant())
            .build();
        button.add_css_class("flat");
        button.set_focus_on_click(false);
        let gesture = gtk4::GestureClick::builder().button(3).build();
        gesture.connect_pressed(clone!(
            #[weak]
            button,
            #[weak]
            appwindow,
            move |_, _, _, _| {
                let popover = gtk4::Popover::new();
                let remove = gtk4::Button::with_label(&gettext("Remove this pen"));
                remove.add_css_class("flat");
                remove.connect_clicked(clone!(
                    #[weak]
                    popover,
                    #[weak]
                    appwindow,
                    move |_| {
                        popover.popdown();
                        gio::prelude::ActionGroupExt::activate_action(
                            &appwindow,
                            "pen-preset-remove",
                            Some(&(i as i32).to_variant()),
                        );
                    }
                ));
                popover.set_child(Some(&remove));
                // not parented to the pen button itself: the button would take the clicks inside the popover
                let Some(anchor) = button.parent().and_then(|favorites| favorites.parent()) else {
                    return;
                };
                if let Some(bounds) = button.compute_bounds(&anchor) {
                    popover.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(
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
        ));
        button.add_controller(gesture);
        container.append(&button);
    }
    let add = gtk4::Button::builder()
        .icon_name("plus-symbolic")
        .tooltip_text(gettext("Save the current pen as favorite"))
        .action_name("win.pen-preset-save")
        .build();
    add.add_css_class("flat");
    add.set_focus_on_click(false);
    container.append(&add);
    let _ = appwindow;
}

// ----- actions

fn string_action(appwindow: &RnAppWindow, name: &str, f: impl Fn(&RnAppWindow, &str) + 'static) {
    let action = gio::SimpleAction::new(name, Some(&String::static_variant_type()));
    action.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, target| {
            let Some(target) = target.and_then(|t| t.get::<String>()) else {
                return;
            };
            f(&appwindow, &target);
        }
    ));
    appwindow.add_action(&action);
}

fn plain_action(appwindow: &RnAppWindow, name: &str, f: impl Fn(&RnAppWindow) + 'static) {
    let action = gio::SimpleAction::new(name, None);
    action.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, _| f(&appwindow)
    ));
    appwindow.add_action(&action);
}

fn with_canvas(
    appwindow: &RnAppWindow,
    f: impl FnOnce(&mut rnote_engine::Engine) -> rnote_engine::WidgetFlags,
) {
    let Some(canvas) = appwindow.active_tab_canvas() else {
        return;
    };
    let widget_flags = f(&mut canvas.engine_mut());
    appwindow.handle_widget_flags(widget_flags, &canvas);
    give_focus_back(&canvas);
}

/// Gives the keyboard focus back to the canvas (after a closing popover took it), so typing continues in the text.
fn give_focus_back(canvas: &crate::RnCanvas) {
    glib::idle_add_local_once(clone!(
        #[weak]
        canvas,
        move || {
            canvas.grab_focus();
        }
    ));
}

fn setup_actions(appwindow: &RnAppWindow) {
    // show or hide the ribbon
    let action_ribbon = gio::SimpleAction::new_stateful("ribbon", None, &true.to_variant());
    appwindow.add_action(&action_ribbon);
    // ink to shape on / off
    let action_shape_recognition =
        gio::SimpleAction::new_stateful("shape-recognition", None, &false.to_variant());
    action_shape_recognition.connect_activate(clone!(
        #[weak]
        appwindow,
        move |action, _| {
            let enabled = {
                let mut config = appwindow.engine_config().write();
                config.pens_config.brush_config.shape_recognition =
                    !config.pens_config.brush_config.shape_recognition;
                config.pens_config.brush_config.shape_recognition
            };
            action.set_state(&enabled.to_variant());
            appwindow.refresh_ui();
        }
    ));
    appwindow.add_action(&action_shape_recognition);

    string_action(appwindow, "text-quick-style", |appwindow, target| {
        let Ok(style) = QuickStyle::from_str(target) else {
            return;
        };
        with_canvas(appwindow, |engine| engine.text_quick_style(style));
    });
    string_action(appwindow, "text-list", |appwindow, target| {
        let kind = match target {
            "numbered" => ListKind::Numbered,
            "checkbox" => ListKind::Checkbox,
            _ => ListKind::Bullet,
        };
        with_canvas(appwindow, |engine| engine.text_paragraph_list(kind));
    });
    string_action(appwindow, "text-script", |appwindow, target| {
        let sup = target == "sup";
        with_canvas(appwindow, |engine| engine.text_selection_toggle_script(sup));
    });
    plain_action(appwindow, "text-strikethrough", |appwindow| {
        with_canvas(appwindow, |engine| {
            engine.text_selection_toggle_attribute(TextAttribute::Strikethrough(true))
        });
    });
    string_action(appwindow, "insert-datetime", |appwindow, target| {
        let kind = match target {
            "time" => DateTimeStamp::Time,
            "both" => DateTimeStamp::DateAndTime,
            _ => DateTimeStamp::Date,
        };
        with_canvas(appwindow, |engine| engine.insert_date_time(kind));
    });
    string_action(appwindow, "insert-table", |appwindow, target| {
        let mut parts = target.split('x').map(|p| p.parse::<u32>().unwrap_or(3));
        let rows = parts.next().unwrap_or(3);
        let cols = parts.next().unwrap_or(3);
        with_canvas(appwindow, |engine| engine.insert_table(rows, cols));
    });
    string_action(appwindow, "insert-template", |appwindow, target| {
        let Some(canvas) = appwindow.active_tab_canvas() else {
            return;
        };
        let title = canvas.doc_title_display();
        let widget_flags = match PageTemplate::from_str(target) {
            Ok(template) => canvas.engine_mut().insert_page_template(template, &title),
            Err(_) => canvas.engine_mut().insert_title_and_date(&title),
        };
        appwindow.handle_widget_flags(widget_flags, &canvas);
    });
    string_action(appwindow, "page-size", |appwindow, target| {
        let (id, landscape) = match target.strip_suffix("-landscape") {
            Some(id) => (id, true),
            None => (target, false),
        };
        let Ok(size) = PageSize::from_str(id) else {
            return;
        };
        with_canvas(appwindow, |engine| engine.set_page_size(size, landscape));
    });
    string_action(appwindow, "tool-style", |appwindow, target| {
        let style = match target {
            "laser" => ToolStyle::Laser,
            "zoom" => ToolStyle::Zoom,
            "offsetcamera" => ToolStyle::OffsetCamera,
            _ => ToolStyle::VerticalSpace,
        };
        appwindow
            .engine_config()
            .write()
            .pens_config
            .tools_config
            .style = style;
        gio::prelude::ActionGroupExt::activate_action(
            appwindow,
            "pen-style",
            Some(&"tools".to_variant()),
        );
        appwindow.refresh_ui();
    });
    string_action(appwindow, "export-fitted", |appwindow, target| {
        let mode = if target == "page-width" {
            FitMode::PageWidth
        } else {
            FitMode::OnePage
        };
        glib::spawn_future_local(clone!(
            #[weak]
            appwindow,
            async move {
                export_fitted(&appwindow, mode).await;
            }
        ));
    });
    plain_action(appwindow, "export-section", |appwindow| {
        glib::spawn_future_local(clone!(
            #[weak]
            appwindow,
            async move {
                export_section(&appwindow).await;
            }
        ));
    });
    plain_action(appwindow, "quick-note", |appwindow| {
        glib::spawn_future_local(clone!(
            #[weak]
            appwindow,
            async move {
                quick_note(&appwindow).await;
            }
        ));
    });
    plain_action(appwindow, "insert-screenshot", |appwindow| {
        glib::spawn_future_local(clone!(
            #[weak]
            appwindow,
            async move {
                insert_screenshot(&appwindow).await;
            }
        ));
    });
    plain_action(appwindow, "open-backups", |appwindow| {
        let dir = crate::utils::onepepe_backup_root();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            error!("Creating the backup folder failed, Err: {e:?}");
        }
        gtk4::FileLauncher::new(Some(&gio::File::for_path(&dir))).launch(
            Some(appwindow),
            None::<&gio::Cancellable>,
            |result| {
                if let Err(e) = result {
                    error!("Opening the backup folder failed, Err: {e:?}");
                }
            },
        );
    });

    // favorite pens
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
            with_canvas(&appwindow, |engine| engine.apply_pen_preset(&preset));
            appwindow.refresh_ui();
        }
    ));
    appwindow.add_action(&action_pen_preset);
    let action_pen_preset_remove =
        gio::SimpleAction::new("pen-preset-remove", Some(&i32::static_variant_type()));
    action_pen_preset_remove.connect_activate(clone!(
        #[weak]
        appwindow,
        move |_, target| {
            let Some(index) = target.and_then(|t| t.get::<i32>()) else {
                return;
            };
            with_presets(|p| p.remove(index as usize));
            save_presets();
            rebuild_favorites(&appwindow);
        }
    ));
    appwindow.add_action(&action_pen_preset_remove);
    plain_action(appwindow, "pen-preset-save", |appwindow| {
        let Some(canvas) = appwindow.active_tab_canvas() else {
            return;
        };
        let count = with_presets(|p| p.0.len());
        let preset =
            canvas
                .engine_ref()
                .current_pen_preset(format!("{} {}", gettext("Pen"), count + 1));
        with_presets(|p| p.add(preset));
        save_presets();
        rebuild_favorites(appwindow);
        appwindow.overlays().dispatch_toast_text(
            &gettext("Saved the current pen as favorite"),
            crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
        );
    });

    appwindow
        .app()
        .set_accels_for_action("win.ribbon", &["<Ctrl>F1"]);
}

/// Exports the drawing as PDF on pages of the page size, shrunk to fit, and warns when the text gets small.
async fn export_fitted(appwindow: &RnAppWindow, mode: FitMode) {
    let Some(canvas) = appwindow.active_tab_canvas() else {
        return;
    };
    let filter = gtk4::FileFilter::new();
    filter.add_mime_type("application/pdf");
    filter.add_suffix("pdf");
    filter.set_name(Some(&gettext("PDF")));
    let filedialog = gtk4::FileDialog::builder()
        .title(gettext("Export fitted to the page"))
        .modal(true)
        .accept_label(gettext("Export"))
        .default_filter(&filter)
        .initial_name(format!("{}.pdf", canvas.doc_title_display()))
        .build();
    if let Some(dir) = appwindow.sidebar().workspacebrowser().dir_list_dir() {
        filedialog.set_initial_folder(Some(&gio::File::for_path(dir)));
    }
    let Ok(file) = filedialog.save_future(Some(appwindow)).await else {
        return;
    };
    let Some(path) = file.path() else {
        return;
    };
    let result = canvas
        .engine_ref()
        .export_doc_as_pdf_fitted(canvas.doc_title_display(), mode);
    match result {
        Ok((bytes, scale)) => {
            if let Err(e) = crate::utils::atomic_save_to_file_future(&path, bytes).await {
                error!("Writing the fitted PDF failed, Err: {e:?}");
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting the PDF failed"));
                return;
            }
            let text = if scale < 0.5 {
                gettext("Exported. Warning: everything is {} times smaller, small handwriting may be hard to read")
                    .replace("{}", &format!("{:.1}", 1.0 / scale))
            } else {
                gettext("Exported the fitted PDF")
            };
            appwindow
                .overlays()
                .dispatch_toast_text(&text, crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT);
        }
        Err(e) => {
            error!("Exporting the fitted PDF failed, Err: {e:?}");
            appwindow
                .overlays()
                .dispatch_toast_error(&gettext("Exporting the PDF failed"));
        }
    }
}

/// The folder of the notebook section shown in the sidebar, or the home folder.
fn section_dir(appwindow: &RnAppWindow) -> std::path::PathBuf {
    appwindow
        .sidebar()
        .workspacebrowser()
        .dir_list_dir()
        .unwrap_or_else(glib::home_dir)
}

/// Exports all the pages of the section (the folder shown in the sidebar) as a single PDF, in name order.
async fn export_section(appwindow: &RnAppWindow) {
    let dir = section_dir(appwindow);
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|e| e == "rnote"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    if files.is_empty() {
        appwindow
            .overlays()
            .dispatch_toast_error(&gettext("This section has no pages"));
        return;
    }
    let section_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| gettext("Section"));
    let filter = gtk4::FileFilter::new();
    filter.add_mime_type("application/pdf");
    filter.add_suffix("pdf");
    filter.set_name(Some(&gettext("PDF")));
    let filedialog = gtk4::FileDialog::builder()
        .title(gettext("Export the whole section as PDF"))
        .modal(true)
        .accept_label(gettext("Export"))
        .default_filter(&filter)
        .initial_name(format!("{section_name}.pdf"))
        .initial_folder(&gio::File::for_path(&dir))
        .build();
    let Ok(file) = filedialog.save_future(Some(appwindow)).await else {
        return;
    };
    let Some(path) = file.path() else {
        return;
    };
    appwindow.overlays().progressbar_start_pulsing();
    let pages: Vec<(String, Vec<u8>)> = files
        .iter()
        .filter_map(|f| {
            let name = f.file_stem()?.to_string_lossy().to_string();
            std::fs::read(f).ok().map(|bytes| (name, bytes))
        })
        .collect();
    let n = pages.len();
    let result =
        rnote_engine::engine::onepepe_notebook::export_pages_as_one_pdf(section_name, pages).await;
    let result = match result {
        Ok(bytes) => crate::utils::atomic_save_to_file_future(&path, bytes).await,
        Err(e) => Err(e),
    };
    appwindow.overlays().progressbar_finish();
    match result {
        Ok(()) => {
            appwindow.overlays().dispatch_toast_text(
                &gettext("Exported {} pages in one PDF").replace("{}", &n.to_string()),
                crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
            );
        }
        Err(e) => {
            error!("Exporting the section failed, Err: {e:?}");
            appwindow
                .overlays()
                .dispatch_toast_error(&gettext("Exporting the PDF failed"));
        }
    }
}

/// A new page in the "Quick notes" section of the notebook, named with the date and time, opened right away
/// with its title and date.
async fn quick_note(appwindow: &RnAppWindow) {
    let dir = section_dir(appwindow).join(gettext("Quick notes"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        error!("Creating the quick notes folder failed, Err: {e:?}");
        appwindow
            .overlays()
            .dispatch_toast_error(&gettext("Creating new page failed"));
        return;
    }
    let now = chrono::Local::now();
    let name = format!("{} {}", gettext("Note"), now.format("%Y-%m-%d %H.%M.%S"));
    let path = dir.join(format!("{name}.rnote"));
    if let Err(e) =
        crate::workspacebrowser::workspaceactions::createpage::write_empty_page(appwindow, &path)
            .await
    {
        error!("Creating the quick note failed, Err: {e:?}");
        appwindow
            .overlays()
            .dispatch_toast_error(&gettext("Creating new page failed"));
        return;
    }
    appwindow
        .open_file_w_dialogs(gio::File::for_path(&path), None, true)
        .await;
    with_canvas(appwindow, |engine| engine.insert_title_and_date(&name));
}

/// Takes a screen clipping through the desktop portal (it lets you choose the area) and inserts it as a picture.
async fn insert_screenshot(appwindow: &RnAppWindow) {
    match screenshot_bytes().await {
        Ok(bytes) => {
            let Some(canvas) = appwindow.active_tab_canvas() else {
                return;
            };
            if let Err(e) = canvas
                .load_in_bitmapimage_bytes(bytes, None, appwindow.respect_borders())
                .await
            {
                error!("Inserting the screen clipping failed, Err: {e:?}");
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Inserting the screen clipping failed"));
            }
        }
        Err(e) => {
            debug!("Screen clipping cancelled or failed, Err: {e:?}");
            appwindow
                .overlays()
                .dispatch_toast_error(&gettext("No screen clipping was taken"));
        }
    }
}

async fn screenshot_bytes() -> anyhow::Result<Vec<u8>> {
    let connection = gio::bus_get_future(gio::BusType::Session).await?;
    let token = format!("onepepe{}", glib::random_int());
    let sender = connection
        .unique_name()
        .ok_or_else(|| anyhow::anyhow!("no D-Bus unique name"))?
        .trim_start_matches(':')
        .replace('.', "_");
    let request_path = format!("/org/freedesktop/portal/desktop/request/{sender}/{token}");

    let (tx, rx) = futures::channel::oneshot::channel::<glib::Variant>();
    let tx = RefCell::new(Some(tx));
    #[allow(deprecated)]
    let subscription = connection.signal_subscribe(
        Some("org.freedesktop.portal.Desktop"),
        Some("org.freedesktop.portal.Request"),
        Some("Response"),
        Some(&request_path),
        None,
        gio::DBusSignalFlags::NONE,
        move |_, _, _, _, _, params| {
            if let Some(tx) = tx.borrow_mut().take() {
                let _ = tx.send(params.clone());
            }
        },
    );

    let options = glib::VariantDict::new(None);
    options.insert("handle_token", &token);
    options.insert("interactive", true);
    let args = ("", options.end()).to_variant();
    let call = connection
        .call_future(
            Some("org.freedesktop.portal.Desktop"),
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Screenshot",
            "Screenshot",
            Some(&args),
            None,
            gio::DBusCallFlags::NONE,
            -1,
        )
        .await;
    let response = match call {
        Ok(_) => rx
            .await
            .map_err(|_| anyhow::anyhow!("no answer from the portal")),
        Err(e) => Err(anyhow::anyhow!("calling the screenshot portal failed: {e}")),
    };
    #[allow(deprecated)]
    connection.signal_unsubscribe(subscription);
    let (code, results) = response?
        .get::<(u32, HashMap<String, glib::Variant>)>()
        .ok_or_else(|| anyhow::anyhow!("unexpected answer from the portal"))?;
    if code != 0 {
        return Err(anyhow::anyhow!("screen clipping cancelled (code {code})"));
    }
    let uri = results
        .get("uri")
        .and_then(|v| v.get::<String>())
        .ok_or_else(|| anyhow::anyhow!("the portal returned no picture"))?;
    let (bytes, _) = gio::File::for_uri(&uri).load_contents_future().await?;
    Ok(bytes.to_vec())
}
