// OnePepe: the ribbon at the top of the window, like the ribbon of OneNote, with the tabs
// File, Home, Insert, Draw and View. It is built in code on top of the window actions (`win.*`).

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, prelude::*, subclass::prelude::*};
use rnote_engine::engine::onepepe_insert::{FitMode, PageSize, PageTemplate};
use rnote_engine::pens::pensconfig::toolsconfig::ToolStyle;
use rnote_engine::pens::typewriter::scripts::{DateTimeStamp, ListKind, QuickStyle};
use rnote_engine::strokes::textstroke::TextAttribute;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, error};

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
    stack.add_titled(&math_page(appwindow), Some("math"), &gettext("Math"));
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
    header.add_css_class("onepepe-tabbar");
    header.append(&switcher);
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);
    header.append(&hide_button);

    let ribbon = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
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
            appwindow,
            #[weak]
            revealer,
            #[weak]
            ribbon,
            move |_| sync_ribbon_visibility(&appwindow, &ribbon, &revealer)
        ));
    }
    appwindow.connect_notify_local(
        Some("focus-mode"),
        clone!(
            #[weak]
            revealer,
            #[weak]
            ribbon,
            move |appwindow, _| sync_ribbon_visibility(appwindow, &ribbon, &revealer)
        ),
    );
    sync_ribbon_visibility(appwindow, &ribbon, &revealer);
    ribbon.add_css_class("onepepe-ribbon");
    stack.add_css_class("onepepe-ribbon-body");
    // OneNote-like look: the title bar and the tabs in the Pepe Flock purple (see data/style.css)
    appwindow.add_css_class("onepepe");
    appwindow
        .main_header()
        .headerbar()
        .add_css_class("onepepe-header");
    appwindow
        .sidebar()
        .headerbar()
        .add_css_class("onepepe-sidebar-header");
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
    crate::statusbar::setup(appwindow);
    crate::crashreport::notify_previous(appwindow);
    crate::optionsbar::setup(appwindow);
    crate::settingslayout::setup(&appwindow.sidebar().settings_panel(), appwindow);
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
    let (mut math, scratch_out) = {
        let config = appwindow.engine_config().read();
        (
            config.pens_config.brush_config.math,
            config.pens_config.brush_config.scratch_out,
        )
    };
    for (name, get) in MATH_TOGGLES {
        if let Some(action) = appwindow
            .lookup_action(name)
            .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
        {
            action.set_state(&(*get(&mut math)).to_variant());
        }
    }
    if let Some(action) = appwindow
        .lookup_action("scratch-out")
        .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
    {
        use rnote_engine::engine::scratch::ScratchOutMode;
        let target = match scratch_out {
            ScratchOutMode::Off => "off",
            ScratchOutMode::MathMode => "math_mode",
            ScratchOutMode::Always => "always",
        };
        action.set_state(&target.to_variant());
    }
    if let (Some(canvas), Some(action)) = (
        appwindow.active_tab_canvas(),
        appwindow
            .lookup_action("doc-guides")
            .and_then(|a| a.downcast::<gio::SimpleAction>().ok()),
    ) {
        action.set_state(
            &canvas
                .engine_ref()
                .document
                .config
                .format
                .show_borders
                .to_variant(),
        );
    }
    if let (Some(canvas), Some(action)) = (
        appwindow.active_tab_canvas(),
        appwindow
            .lookup_action("ruler-infinite")
            .and_then(|a| a.downcast::<gio::SimpleAction>().ok()),
    ) {
        action.set_state(&canvas.engine_ref().ruler_infinite().to_variant());
    }
    crate::favorites::refresh(appwindow);
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
        &[(gettext("Quote"), "quote"), (gettext("Code"), "code")],
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
                    small_button(
                        "view-list-bullet-symbolic",
                        &gettext("Bullet list"),
                        "win.text-list",
                        s("bullet"),
                    )
                    .upcast(),
                    small_button(
                        "view-list-ordered-symbolic",
                        &gettext("Numbered list"),
                        "win.text-list",
                        s("numbered"),
                    )
                    .upcast(),
                    text_button("☐", &gettext("Checkbox"), "win.text-list", s("checkbox")).upcast(),
                ],
            ])],
        ),
        group(&gettext("Styles"), &{
            let mut widgets = crate::textstyles::gallery(&styles_popover);
            let edit = gtk4::MenuButton::builder()
                .icon_name(available_icon("document-edit-symbolic"))
                .tooltip_text(gettext("Edit text styles"))
                .popover(&crate::textstyles::edit_popover(appwindow))
                .valign(gtk4::Align::Center)
                .build();
            edit.add_css_class("flat");
            edit.set_focus_on_click(false);
            widgets.push(edit.upcast());
            widgets
        }),
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
    let favorites = crate::favorites::widget(appwindow);

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
                big_button(
                    "pen-tools-offsetcameratool-symbolic",
                    &gettext("Hand"),
                    "win.tool-style",
                    s("offsetcamera"),
                )
                .upcast(),
            ],
        ),
        group(&gettext("Colors"), &[colors_in_ribbon(appwindow).upcast()]),
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
                    "shapebuilder-line-symbolic",
                    &gettext("Infinite ruler"),
                    "win.ruler-infinite",
                    None,
                )
                .upcast(),
                big_menu(
                    "object-rotate-left-symbolic",
                    &gettext("Angle"),
                    &ruler_angle_popover(appwindow),
                )
                .upcast(),
                big_toggle(
                    "onepepe-ink-to-shape-symbolic",
                    &gettext("Ink to Shape"),
                    "win.shape-recognition",
                    None,
                )
                .upcast(),
            ],
        ),
    ])
}

fn math_page(_appwindow: &RnAppWindow) -> gtk4::ScrolledWindow {
    let scratch_popover = list_popover(
        "win.scratch-out",
        &[
            (gettext("Never"), "off"),
            (gettext("Only in math mode"), "math_mode"),
            (gettext("Always, with the pen"), "always"),
        ],
    );
    page(&[
        group(
            &gettext("Math Mode"),
            &[
                big_toggle(
                    "onepepe-math-symbolic",
                    &gettext("Math Mode"),
                    "win.math-mode",
                    None,
                )
                .upcast(),
                big_button(
                    "edit-select-all-symbolic",
                    &gettext("Recognize selection"),
                    "win.math-recognize-selection",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Calculate"),
            &[
                big_toggle(
                    "accessories-calculator-symbolic",
                    &gettext("Result of \"=\""),
                    "win.math-equals",
                    None,
                )
                .upcast(),
                big_toggle(
                    "format-text-italic-symbolic",
                    &gettext("Variables"),
                    "win.math-variables",
                    None,
                )
                .upcast(),
                big_toggle(
                    "object-select-symbolic",
                    &gettext("Check steps"),
                    "win.math-check",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Selection"),
            &[
                big_button(
                    "edit-copy-symbolic",
                    &gettext("Copy as LaTeX"),
                    "win.math-copy-latex",
                    None,
                )
                .upcast(),
                big_button(
                    "media-playlist-shuffle-symbolic",
                    &gettext("Plot"),
                    "win.math-plot",
                    None,
                )
                .upcast(),
            ],
        ),
        group(
            &gettext("Scratch out"),
            &[big_menu(
                "edit-clear-symbolic",
                &gettext("Zigzag erases"),
                &scratch_popover,
            )
            .upcast()],
        ),
        group(
            &gettext("Your handwriting"),
            &[
                big_button(
                    "document-new-symbolic",
                    &gettext("New training"),
                    "win.math-training-new",
                    None,
                )
                .upcast(),
                big_button(
                    "document-open-symbolic",
                    &gettext("Open training"),
                    "win.math-training-open",
                    None,
                )
                .upcast(),
                big_button(
                    "restore-symbolic",
                    &gettext("Retrain"),
                    "win.math-retrain",
                    None,
                )
                .upcast(),
            ],
        ),
    ])
}

fn view_page(appwindow: &RnAppWindow) -> gtk4::ScrolledWindow {
    let size_popover = page_size_popover();
    let lines_popover = crate::lines::popover(appwindow);
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
                {
                    let toggle = big_toggle(
                        "zoom-original-symbolic",
                        &gettext("Vector zoom"),
                        "win.vector-zoom",
                        None,
                    );
                    toggle.set_tooltip_text(Some(&gettext(
                        "Zoom in up to 100 000 % (normally up to 1600 %). Only on screen: the exports do not change.",
                    )));
                    toggle
                }
                .upcast(),
                big_button(
                    "pen-tools-offsetcameratool-symbolic",
                    &gettext("Hand"),
                    "win.tool-style",
                    s("offsetcamera"),
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
            &gettext("Background and lines"),
            &[
                big_menu("view-grid-symbolic", &gettext("Lines"), &lines_popover).upcast(),
                big_toggle(
                    "shapebuilder-line-symbolic",
                    &gettext("Infinite ruler"),
                    "win.ruler-infinite",
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
                big_toggle(
                    "view-grid-symbolic",
                    &gettext("Guides"),
                    "win.doc-guides",
                    None,
                )
                .upcast(),
                big_toggle(
                    "weather-clear-night-symbolic",
                    &gettext("Dark canvas"),
                    "win.dark-canvas",
                    None,
                )
                .upcast(),
            ],
        ),
    ])
}

/// Moves the color picker from its floating bar over the canvas into the ribbon (Draw tab), like the pen colors of
/// OneNote. The canvas stays clean.
fn colors_in_ribbon(appwindow: &RnAppWindow) -> gtk4::Box {
    let slot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    slot.set_valign(gtk4::Align::Center);
    let colorpicker = appwindow.overlays().colorpicker();
    appwindow
        .overlays()
        .imp()
        .toolbar_overlay
        .remove_overlay(&colorpicker);
    colorpicker.remove_css_class("overlay_toolbar");
    colorpicker.set_halign(gtk4::Align::Start);
    colorpicker.set_valign(gtk4::Align::Center);
    colorpicker.set_margin_top(0);
    colorpicker.set_margin_bottom(0);
    colorpicker.set_margin_start(0);
    colorpicker.set_margin_end(0);
    // the settings may set a margin for the floating bar: not wanted inside the ribbon
    colorpicker.connect_margin_top_notify(|w| {
        if w.margin_top() != 0 {
            w.set_margin_top(0);
        }
    });
    slot.append(&colorpicker);
    slot
}

/// The floating pen bar at the bottom of the canvas repeats the ribbon: it is shown only when the ribbon is hidden.
/// In focus mode the ribbon is hidden as well, like the full page view of OneNote.
fn sync_ribbon_visibility(appwindow: &RnAppWindow, ribbon: &gtk4::Box, revealer: &gtk4::Revealer) {
    let shown = appwindow
        .lookup_action("ribbon")
        .and_then(|a| a.state())
        .and_then(|s| s.get::<bool>())
        .unwrap_or(true);
    let focus_mode = appwindow.property::<bool>("focus-mode");
    ribbon.set_visible(!focus_mode);
    revealer.set_reveal_child(shown);
    appwindow
        .overlays()
        .penpicker()
        .set_visible(!shown && !focus_mode);
}

/// The popover of the page sizes: one row per size, portrait and landscape side by side.
/// The exact angle of the ruler: typed, or one of the usual ones. Turning the handle is free too (Shift: 15° steps).
fn ruler_angle_popover(appwindow: &RnAppWindow) -> gtk4::Popover {
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    row.append(&gtk4::Label::new(Some(&gettext("Angle of the ruler"))));
    let spin = gtk4::SpinButton::with_range(0.0, 180.0, 1.0);
    spin.set_digits(1);
    spin.set_wrap(true);
    row.append(&spin);
    row.append(&gtk4::Label::new(Some("°")));
    content.append(&row);
    let quick = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    quick.add_css_class("linked");
    for degrees in [0.0, 30.0, 45.0, 60.0, 90.0, 135.0] {
        let button = gtk4::Button::with_label(&format!("{degrees:.0}°"));
        button.connect_clicked(clone!(
            #[weak]
            spin,
            move |_| spin.set_value(degrees)
        ));
        quick.append(&button);
    }
    content.append(&quick);
    let hint = gtk4::Label::new(Some(&gettext(
        "You can also turn it by its circle; with Shift, in steps of 15°.",
    )));
    hint.add_css_class("dim-label");
    hint.set_wrap(true);
    hint.set_max_width_chars(32);
    content.append(&hint);
    let popover = gtk4::Popover::builder().child(&content).build();
    let filling = std::rc::Rc::new(std::cell::Cell::new(false));
    popover.connect_show(clone!(
        #[weak]
        appwindow,
        #[weak]
        spin,
        #[strong]
        filling,
        move |_| {
            let angle = appwindow
                .active_tab_canvas()
                .and_then(|c| c.engine_ref().ruler_angle_degrees())
                .unwrap_or(0.0);
            filling.set(true);
            spin.set_value(angle);
            filling.set(false);
        }
    ));
    spin.connect_value_changed(clone!(
        #[weak]
        appwindow,
        #[strong]
        filling,
        move |spin| {
            if filling.get() {
                return;
            }
            with_canvas(&appwindow, |engine| {
                engine.set_ruler_angle_degrees(spin.value())
            });
            appwindow.refresh_ui();
        }
    ));
    popover
}

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

/// The math options with a toggle in the ribbon: (action, option).
type MathToggle = (
    &'static str,
    fn(&mut rnote_engine::engine::math::MathOptions) -> &mut bool,
);
const MATH_TOGGLES: [MathToggle; 3] = [
    ("math-equals", |o| &mut o.equals),
    ("math-variables", |o| &mut o.variables),
    ("math-check", |o| &mut o.check),
];

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
    // OnePepe: what the math mode writes next to the lines (result of "=", variables, step check)
    for (name, get) in MATH_TOGGLES {
        let action = gio::SimpleAction::new_stateful(name, None, &false.to_variant());
        action.connect_activate(clone!(
            #[weak]
            appwindow,
            move |action, _| {
                let enabled = {
                    let mut config = appwindow.engine_config().write();
                    let option = get(&mut config.pens_config.brush_config.math);
                    *option = !*option;
                    *option
                };
                action.set_state(&enabled.to_variant());
                if let Some(canvas) = appwindow.active_tab_canvas() {
                    crate::mathhelper::schedule_page(&appwindow, &canvas);
                }
                appwindow.refresh_ui();
            }
        ));
        appwindow.add_action(&action);
    }
    // OnePepe: when a zigzag erases
    let action_scratch_out = gio::SimpleAction::new_stateful(
        "scratch-out",
        Some(&String::static_variant_type()),
        &String::from("math_mode").to_variant(),
    );
    action_scratch_out.connect_activate(clone!(
        #[weak]
        appwindow,
        move |action, target| {
            let Some(target) = target.and_then(|t| t.get::<String>()) else {
                return;
            };
            use rnote_engine::engine::scratch::ScratchOutMode;
            let mode = match target.as_str() {
                "off" => ScratchOutMode::Off,
                "always" => ScratchOutMode::Always,
                _ => ScratchOutMode::MathMode,
            };
            appwindow
                .engine_config()
                .write()
                .pens_config
                .brush_config
                .scratch_out = mode;
            action.set_state(&target.to_variant());
            appwindow.refresh_ui();
        }
    ));
    appwindow.add_action(&action_scratch_out);
    // dark canvas in the dark theme (only on screen), on by default
    let action_dark_canvas = gio::SimpleAction::new_stateful(
        "dark-canvas",
        None,
        &crate::prefs::get_bool("lienzo-oscuro", true).to_variant(),
    );
    action_dark_canvas.connect_activate(clone!(
        #[weak]
        appwindow,
        move |action, _| {
            let enabled = !crate::prefs::get_bool("lienzo-oscuro", true);
            crate::prefs::set("lienzo-oscuro", enabled.into());
            action.set_state(&enabled.to_variant());
            if let Some(canvas) = appwindow.active_tab_canvas() {
                canvas.queue_draw();
            }
        }
    ));
    appwindow.add_action(&action_dark_canvas);
    // the page guides (the borders of the pages) of the document, saved in the document as in Rnote
    let action_guides = gio::SimpleAction::new_stateful("doc-guides", None, &false.to_variant());
    action_guides.connect_activate(clone!(
        #[weak]
        appwindow,
        move |action, _| {
            let Some(canvas) = appwindow.active_tab_canvas() else {
                return;
            };
            let show = {
                let mut engine = canvas.engine_mut();
                let format = &mut engine.document.config.format;
                format.show_borders = !format.show_borders;
                format.show_borders
            };
            action.set_state(&show.to_variant());
            canvas.set_unsaved_changes(true);
            canvas.queue_draw();
        }
    ));
    appwindow.add_action(&action_guides);
    // OnePepe: vector zoom, up to 100 000 % (only on screen); without it, up to 1600 %
    rnote_engine::Camera::set_vector_zoom(crate::prefs::get_bool("zoom-vectorial", false));
    let action_vector_zoom = gio::SimpleAction::new_stateful(
        "vector-zoom",
        None,
        &rnote_engine::Camera::vector_zoom().to_variant(),
    );
    action_vector_zoom.connect_activate(clone!(
        #[weak]
        appwindow,
        move |action, _| {
            let on = !rnote_engine::Camera::vector_zoom();
            rnote_engine::Camera::set_vector_zoom(on);
            crate::prefs::set("zoom-vectorial", on.into());
            action.set_state(&on.to_variant());
            if on {
                return;
            }
            // back to the normal limit
            for page in appwindow.tabs_snapshot() {
                let Some(wrapper) = page
                    .child()
                    .downcast_ref::<crate::RnCanvasWrapper>()
                    .cloned()
                else {
                    continue;
                };
                let canvas = wrapper.canvas();
                if canvas.engine_ref().camera.zoom() <= rnote_engine::Camera::ZOOM_MAX {
                    continue;
                }
                let center = canvas.engine_ref().camera.viewport_center();
                let mut widget_flags = canvas
                    .engine_mut()
                    .zoom_w_timeout(rnote_engine::Camera::ZOOM_MAX);
                widget_flags |= canvas.engine_mut().camera.set_viewport_center(center);
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        }
    ));
    appwindow.add_action(&action_vector_zoom);
    // the infinite ruler: a line across the whole canvas (kept in the hidden file of the page)
    let action_ruler_infinite =
        gio::SimpleAction::new_stateful("ruler-infinite", None, &false.to_variant());
    action_ruler_infinite.connect_activate(clone!(
        #[weak]
        appwindow,
        move |action, _| {
            let Some(canvas) = appwindow.active_tab_canvas() else {
                return;
            };
            let infinite = !canvas.engine_ref().ruler_infinite();
            let widget_flags = canvas.engine_mut().set_ruler_infinite(infinite);
            appwindow.handle_widget_flags(widget_flags, &canvas);
            action.set_state(&infinite.to_variant());
            appwindow.refresh_ui();
        }
    ));
    appwindow.add_action(&action_ruler_infinite);

    string_action(appwindow, "text-quick-style", |appwindow, target| {
        let Ok(style) = QuickStyle::from_str(target) else {
            return;
        };
        let styles = crate::textstyles::current();
        with_canvas(appwindow, |engine| engine.text_quick_style(style, &styles));
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
    plain_action(appwindow, "math-retrain", |appwindow| {
        crate::mathhelper::retrain(appwindow);
    });
    plain_action(appwindow, "math-training-new", |appwindow| {
        crate::entreno::new_training(appwindow);
    });
    plain_action(appwindow, "math-training-open", |appwindow| {
        crate::entreno::open_training(appwindow);
    });
    plain_action(appwindow, "math-recognize-selection", |appwindow| {
        crate::mathhelper::recognize_selection(appwindow);
    });
    plain_action(appwindow, "math-copy-latex", |appwindow| {
        crate::mathhelper::copy_latex(appwindow);
    });
    plain_action(appwindow, "math-plot", |appwindow| {
        crate::mathhelper::plot_selection(appwindow);
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

    crate::favorites::setup_actions(appwindow);
    crate::textstyles::setup_accels(appwindow);

    appwindow
        .app()
        .set_accels_for_action("win.ribbon", &["<Ctrl>F1"]);
    // the hand, to move around the page without holding a key (next to the tools, Alt+1…Alt+6)
    appwindow
        .app()
        .set_accels_for_action("win.tool-style::offsetcamera", &["<Alt>7", "<Alt>KP_7"]);
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
    crate::workspacebrowser::tree::current_section(appwindow).unwrap_or_else(glib::home_dir)
}

/// The pages of a section in the order of the tree: each page followed by its subpages.
fn section_pages_in_order(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    use rnote_engine::onepepe_index::FolderIndex;
    let entries = FolderIndex::read_entries(dir);
    let index = FolderIndex::load(dir);
    let mut out = vec![];
    let mut stack: Vec<String> = index
        .children(&entries, None)
        .into_iter()
        .rev()
        .map(|e| e.name)
        .collect();
    while let Some(name) = stack.pop() {
        if name.ends_with(".rnote") {
            out.push(dir.join(&name));
        }
        for child in index.children(&entries, Some(&name)).into_iter().rev() {
            stack.push(child.name);
        }
    }
    out
}

/// Exports all the pages of the section (the folder shown in the sidebar) as a single PDF, in name order.
async fn export_section(appwindow: &RnAppWindow) {
    let dir = section_dir(appwindow);
    // in the order of the tree (subpages after their page)
    let files: Vec<std::path::PathBuf> = section_pages_in_order(&dir);
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
