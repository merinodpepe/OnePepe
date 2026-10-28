// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: «Background and lines» of the View tab: type, spacing (mm or px, with presets), width, colors and the
// margin line, with a live preview on the open page. «Apply» keeps it for the page, or for all the pages of the
// section (the other pages are rewritten, after a backup copy). Closing without applying goes back.
// Named presets of the user are kept in ~/.config/onepepe/ajustes.json.

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gdk, glib, glib::clone, prelude::*};
use rnote_compose::Color;
use rnote_engine::document::background::PatternStyle;
use rnote_engine::engine::onepepe_lines::{LinesSettings, rewrite_page_lines};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tracing::error;

const PATTERNS: [PatternStyle; 6] = [
    PatternStyle::None,
    PatternStyle::Lines,
    PatternStyle::Grid,
    PatternStyle::Dots,
    PatternStyle::IsometricGrid,
    PatternStyle::IsometricDots,
];

fn pattern_names() -> [String; 6] {
    [
        gettext("Blank"),
        gettext("Lines"),
        gettext("Grid"),
        gettext("Dots"),
        gettext("Isometric grid"),
        gettext("Isometric dots"),
    ]
}

fn preset_name(id: &str) -> String {
    match id {
        "blank" => gettext("Blank"),
        "ruled-7" => gettext("Ruled 7 mm"),
        "ruled-8" => gettext("Ruled 8.7 mm"),
        "grid-5" => gettext("Grid 5 mm"),
        "dots-5" => gettext("Dots 5 mm"),
        other => other.to_string(),
    }
}

fn user_presets() -> Vec<(String, LinesSettings)> {
    crate::prefs::get("presets-fondo")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

fn to_rgba(c: Color) -> gdk::RGBA {
    gdk::RGBA::new(c.r as f32, c.g as f32, c.b as f32, c.a as f32)
}

fn from_rgba(c: &gdk::RGBA) -> Color {
    Color::new(
        c.red() as f64,
        c.green() as f64,
        c.blue() as f64,
        c.alpha() as f64,
    )
}

/// The editors of the popover, to read and write the settings.
struct Editors {
    pattern: gtk4::DropDown,
    spacing: gtk4::SpinButton,
    in_mm: gtk4::DropDown,
    width: gtk4::SpinButton,
    color: gtk4::ColorDialogButton,
    background: gtk4::ColorDialogButton,
    margin_on: gtk4::Switch,
    margin: gtk4::SpinButton,
    whole_section: gtk4::CheckButton,
    /// While the editors are being filled, they must not apply anything.
    filling: Cell<bool>,
}

impl Editors {
    fn read(&self) -> LinesSettings {
        let mm = self.in_mm.selected() == 0;
        let spacing = self.spacing.value();
        LinesSettings {
            pattern: PATTERNS[(self.pattern.selected() as usize).min(PATTERNS.len() - 1)],
            spacing: if mm {
                LinesSettings::mm_to_px(spacing)
            } else {
                spacing
            },
            line_width: self.width.value(),
            color: from_rgba(&self.color.rgba()),
            background_color: from_rgba(&self.background.rgba()),
            margin: self
                .margin_on
                .is_active()
                .then(|| LinesSettings::mm_to_px(self.margin.value())),
        }
    }

    fn write(&self, s: &LinesSettings) {
        self.filling.set(true);
        self.pattern
            .set_selected(PATTERNS.iter().position(|p| *p == s.pattern).unwrap_or(0) as u32);
        let mm = self.in_mm.selected() == 0;
        self.spacing.set_value(if mm {
            (LinesSettings::px_to_mm(s.spacing) * 10.0).round() / 10.0
        } else {
            s.spacing.round()
        });
        self.width.set_value(s.line_width);
        self.color.set_rgba(&to_rgba(s.color));
        self.background.set_rgba(&to_rgba(s.background_color));
        self.margin_on.set_active(s.margin.is_some());
        self.margin.set_value(
            (LinesSettings::px_to_mm(s.margin.unwrap_or(LinesSettings::mm_to_px(25.0))) * 10.0)
                .round()
                / 10.0,
        );
        self.filling.set(false);
    }
}

/// The popover of «Background and lines».
pub(crate) fn popover(appwindow: &RnAppWindow) -> gtk4::Popover {
    let popover = gtk4::Popover::new();
    let grid = gtk4::Grid::builder()
        .row_spacing(8)
        .column_spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let title = gtk4::Label::new(Some(&gettext("Background and lines")));
    title.add_css_class("heading");
    title.set_halign(gtk4::Align::Start);
    grid.attach(&title, 0, 0, 3, 1);

    // presets
    let presets = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(3)
        .column_spacing(4)
        .row_spacing(4)
        .build();
    grid.attach(&presets, 0, 1, 3, 1);

    let label = |text: String| {
        let l = gtk4::Label::new(Some(&text));
        l.set_halign(gtk4::Align::Start);
        l
    };
    let names = pattern_names();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let editors = Rc::new(Editors {
        pattern: gtk4::DropDown::from_strings(&names),
        spacing: gtk4::SpinButton::with_range(1.0, 400.0, 0.5),
        in_mm: gtk4::DropDown::from_strings(&["mm", "px"]),
        width: gtk4::SpinButton::with_range(
            LinesSettings::WIDTH_MIN,
            LinesSettings::WIDTH_MAX,
            0.1,
        ),
        color: gtk4::ColorDialogButton::new(Some(gtk4::ColorDialog::new())),
        background: gtk4::ColorDialogButton::new(Some(gtk4::ColorDialog::new())),
        margin_on: gtk4::Switch::builder().valign(gtk4::Align::Center).build(),
        margin: gtk4::SpinButton::with_range(1.0, 200.0, 1.0),
        whole_section: gtk4::CheckButton::with_label(&gettext("All the pages of the section")),
        filling: Cell::new(false),
    });
    editors.spacing.set_digits(1);
    editors.width.set_digits(1);
    editors.margin.set_digits(0);
    let this_page = gtk4::CheckButton::with_label(&gettext("This page"));
    this_page.set_active(true);
    editors.whole_section.set_group(Some(&this_page));

    grid.attach(&label(gettext("Type")), 0, 2, 1, 1);
    grid.attach(&editors.pattern, 1, 2, 2, 1);
    grid.attach(&label(gettext("Spacing")), 0, 3, 1, 1);
    grid.attach(&editors.spacing, 1, 3, 1, 1);
    grid.attach(&editors.in_mm, 2, 3, 1, 1);
    grid.attach(&label(gettext("Width")), 0, 4, 1, 1);
    grid.attach(&editors.width, 1, 4, 2, 1);
    grid.attach(&label(gettext("Line color")), 0, 5, 1, 1);
    grid.attach(&editors.color, 1, 5, 2, 1);
    grid.attach(&label(gettext("Paper color")), 0, 6, 1, 1);
    grid.attach(&editors.background, 1, 6, 2, 1);
    let margin_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    margin_box.append(&editors.margin_on);
    margin_box.append(&editors.margin);
    margin_box.append(&gtk4::Label::new(Some("mm")));
    grid.attach(&label(gettext("Margin line")), 0, 7, 1, 1);
    grid.attach(&margin_box, 1, 7, 2, 1);
    grid.attach(&label(gettext("Apply to")), 0, 8, 1, 1);
    let scope = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    scope.append(&this_page);
    scope.append(&editors.whole_section);
    grid.attach(&scope, 1, 8, 2, 1);

    let buttons = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    buttons.set_halign(gtk4::Align::End);
    let save_preset = gtk4::MenuButton::builder()
        .label(gettext("Save as…"))
        .tooltip_text(gettext("Save these settings with a name"))
        .build();
    let reset = gtk4::Button::with_label(&gettext("Reset"));
    let apply = gtk4::Button::with_label(&gettext("Apply"));
    apply.add_css_class("suggested-action");
    buttons.append(&save_preset);
    buttons.append(&reset);
    buttons.append(&apply);
    grid.attach(&buttons, 0, 9, 3, 1);
    popover.set_child(Some(&grid));

    // the settings when the popover opened, to go back if it closes without «Apply»
    let original: Rc<RefCell<Option<LinesSettings>>> = Rc::new(RefCell::new(None));
    let applied = Rc::new(Cell::new(false));

    let preview = Rc::new(clone!(
        #[weak]
        appwindow,
        #[strong]
        editors,
        move || {
            if editors.filling.get() {
                return;
            }
            let settings = editors.read();
            if let Some(canvas) = appwindow.active_tab_canvas() {
                let widget_flags = canvas.engine_mut().apply_lines_settings(&settings);
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        }
    ));
    for spin in [&editors.spacing, &editors.width, &editors.margin] {
        spin.connect_value_changed(clone!(
            #[strong]
            preview,
            move |_| preview()
        ));
    }
    editors.pattern.connect_selected_notify(clone!(
        #[strong]
        preview,
        move |_| preview()
    ));
    for button in [&editors.color, &editors.background] {
        button.connect_rgba_notify(clone!(
            #[strong]
            preview,
            move |_| preview()
        ));
    }
    editors.margin_on.connect_active_notify(clone!(
        #[strong]
        preview,
        #[strong]
        editors,
        move |switch| {
            editors.margin.set_sensitive(switch.is_active());
            preview()
        }
    ));
    // switching mm/px converts the value shown
    let was_mm = Rc::new(Cell::new(true));
    editors.in_mm.connect_selected_notify(clone!(
        #[strong]
        editors,
        #[strong]
        was_mm,
        move |in_mm| {
            let mm = in_mm.selected() == 0;
            if mm == was_mm.get() {
                return;
            }
            was_mm.set(mm);
            let px = if mm {
                editors.spacing.value()
            } else {
                LinesSettings::mm_to_px(editors.spacing.value())
            };
            editors.filling.set(true);
            editors.spacing.set_value(if mm {
                (LinesSettings::px_to_mm(px) * 10.0).round() / 10.0
            } else {
                px.round()
            });
            editors.filling.set(false);
        }
    ));

    let fill_presets = Rc::new(clone!(
        #[weak]
        presets,
        #[strong]
        editors,
        #[strong]
        preview,
        move || {
            presets.remove_all();
            let all: Vec<(String, LinesSettings)> = LinesSettings::presets()
                .into_iter()
                .map(|(id, s)| (preset_name(id), s))
                .chain(user_presets())
                .collect();
            for (name, settings) in all {
                let button = gtk4::Button::with_label(&name);
                button.add_css_class("flat");
                button.connect_clicked(clone!(
                    #[strong]
                    editors,
                    #[strong]
                    preview,
                    move |_| {
                        editors.write(&settings);
                        preview();
                    }
                ));
                presets.append(&button);
            }
        }
    ));
    fill_presets();

    // «Save as…»: a name for the current settings
    let name_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    name_box.set_margin_top(6);
    name_box.set_margin_bottom(6);
    name_box.set_margin_start(6);
    name_box.set_margin_end(6);
    let name_entry = gtk4::Entry::builder()
        .placeholder_text(gettext("Name"))
        .build();
    let name_ok = gtk4::Button::with_label(&gettext("Save"));
    name_box.append(&name_entry);
    name_box.append(&name_ok);
    let name_popover = gtk4::Popover::builder().child(&name_box).build();
    save_preset.set_popover(Some(&name_popover));
    name_ok.connect_clicked(clone!(
        #[strong]
        editors,
        #[strong]
        fill_presets,
        #[weak]
        name_entry,
        #[weak]
        name_popover,
        move |_| {
            let name = name_entry.text().trim().to_string();
            if name.is_empty() {
                return;
            }
            let mut list = user_presets();
            list.retain(|(n, _)| *n != name);
            list.push((name, editors.read()));
            if let Ok(value) = serde_json::to_value(&list) {
                crate::prefs::set("presets-fondo", value);
            }
            name_entry.set_text("");
            name_popover.popdown();
            fill_presets();
        }
    ));

    reset.connect_clicked(clone!(
        #[strong]
        editors,
        #[strong]
        preview,
        move |_| {
            editors.write(&LinesSettings::default());
            preview();
        }
    ));

    popover.connect_show(clone!(
        #[weak]
        appwindow,
        #[strong]
        editors,
        #[strong]
        original,
        #[strong]
        applied,
        move |_| {
            applied.set(false);
            let current = appwindow
                .active_tab_canvas()
                .map(|c| c.engine_ref().lines_settings())
                .unwrap_or_default();
            original.replace(Some(current));
            editors.write(&current);
            editors.margin.set_sensitive(current.margin.is_some());
        }
    ));
    popover.connect_closed(clone!(
        #[weak]
        appwindow,
        #[strong]
        original,
        #[strong]
        applied,
        move |_| {
            // closed without «Apply»: back to how it was
            if applied.get() {
                return;
            }
            if let (Some(settings), Some(canvas)) =
                (original.borrow_mut().take(), appwindow.active_tab_canvas())
            {
                let widget_flags = canvas.engine_mut().apply_lines_settings(&settings);
                appwindow.handle_widget_flags(widget_flags, &canvas);
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
        #[strong]
        applied,
        move |_| {
            applied.set(true);
            let settings = editors.read();
            if let Some(canvas) = appwindow.active_tab_canvas() {
                let widget_flags = canvas.engine_mut().apply_lines_settings(&settings);
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
            let whole_section = editors.whole_section.is_active();
            popover.popdown();
            if whole_section {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move { apply_to_section(&appwindow, settings).await }
                ));
            }
        }
    ));
    popover
}

/// Applies the settings to all the pages of the section of the open page: the open ones live, the others
/// rewritten (after a backup copy of each one).
async fn apply_to_section(appwindow: &RnAppWindow, settings: LinesSettings) {
    let Some(dir) = crate::workspacebrowser::tree::current_section(appwindow) else {
        return;
    };
    let open: Vec<(std::path::PathBuf, crate::RnCanvas)> = appwindow
        .tabs_snapshot()
        .into_iter()
        .filter_map(|page| page.child().downcast::<crate::RnCanvasWrapper>().ok())
        .map(|w| w.canvas())
        .filter_map(|c| c.output_file().and_then(|f| f.path()).map(|p| (p, c)))
        .collect();
    let pages: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map(|r| {
            r.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|e| e == "rnote"))
                .collect()
        })
        .unwrap_or_default();
    let (mut changed, mut failed) = (0, 0);
    for page in pages {
        if let Some((_, canvas)) = open.iter().find(|(p, _)| *p == page) {
            let widget_flags = canvas.engine_mut().apply_lines_settings(&settings);
            appwindow.handle_widget_flags(widget_flags, canvas);
            changed += 1;
            continue;
        }
        match rewrite_closed_page(&page, settings).await {
            Ok(()) => changed += 1,
            Err(e) => {
                error!("Could not change the lines of {page:?}, Err: {e:?}");
                failed += 1;
            }
        }
    }
    let mut message =
        gettext("Lines changed in {} pages of the section").replace("{}", &changed.to_string());
    if failed > 0 {
        message += &gettext(" ({} could not be changed)").replace("{}", &failed.to_string());
    }
    appwindow
        .overlays()
        .dispatch_toast_text(&message, crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT);
}

async fn rewrite_closed_page(
    page: &std::path::Path,
    settings: LinesSettings,
) -> anyhow::Result<()> {
    rnote_engine::backup::backup_before_save(
        page,
        &crate::utils::onepepe_backup_root(),
        rnote_engine::backup::KEEP_DEFAULT,
        std::time::Duration::ZERO,
        chrono::Local::now().naive_local(),
    )?;
    let bytes = async_fs::read(page).await?;
    let sidecar_path = rnote_engine::engine::math::math_sidecar_path(page);
    let sidecar = async_fs::read_to_string(&sidecar_path).await.ok();
    let file_name = page
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let (bytes, sidecar) = rewrite_page_lines(bytes, file_name, sidecar, settings).await?;
    // never half written: a temporary file, then renamed over the page
    let tmp = page.with_extension("rnote.tmp");
    async_fs::write(&tmp, bytes).await?;
    async_fs::rename(&tmp, page).await?;
    async_fs::write(&sidecar_path, sidecar).await?;
    Ok(())
}
