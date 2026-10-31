// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: training mode. Write known expressions by hand to teach your handwriting to the math recognizer.
//
// A window shows one expression at a time (drawn from its LaTeX by the math helper) and a box to write it in. What
// you write is saved in a training file (`*.entreno.json`, not a `.rnote`) with the strokes and their times, so you
// can stop and go on another day. «Use for training» gives it to the math helper, which cuts each expression into
// symbols knowing what it says and keeps them as examples of your handwriting (tools/onepepe-math/onepepe_math/
// entreno.py). Nothing leaves your computer.

// Imports
use crate::RnAppWindow;
use adw::prelude::*;
use base64::Engine as _;
use gettextrs::gettext;
use gtk4::{gdk, gio, glib, glib::clone};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The file suffix of the training files.
pub(crate) const SUFFIX: &str = ".entreno.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    id: String,
    kind: String,
    lines: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Answer {
    #[serde(default)]
    strokes: Vec<Vec<[f64; 2]>>,
    /// Start and end time of each stroke, in seconds since the first one.
    #[serde(default)]
    times: Vec<[f64; 2]>,
    /// Width and height of the box where it was written (the guide lines split its height).
    #[serde(default)]
    size: [f64; 2],
    #[serde(default)]
    lines: usize,
    /// Height of each guide line (the strokes of each step are in its band).
    #[serde(default)]
    line_height: f64,
    #[serde(default)]
    written: String,
    #[serde(default)]
    skipped: bool,
    #[serde(default)]
    imported: bool,
}

/// The training file. Its format is described in `onepepe_math/entreno.py` (`new_session`).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Session {
    format: String,
    version: u32,
    #[serde(default)]
    app_version: String,
    #[serde(default)]
    set: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    current: usize,
    items: Vec<Item>,
    #[serde(default)]
    answers: BTreeMap<String, Answer>,
}

impl Session {
    fn written(&self) -> usize {
        self.answers
            .values()
            .filter(|a| !a.skipped && !a.strokes.is_empty())
            .count()
    }

    fn to_be_imported(&self) -> usize {
        self.answers
            .values()
            .filter(|a| !a.skipped && !a.imported && !a.strokes.is_empty())
            .count()
    }
}

/// What is being written now.
struct State {
    path: PathBuf,
    session: Session,
    strokes: Vec<Vec<[f64; 2]>>,
    times: Vec<[f64; 2]>,
    /// When the first stroke of the current expression started.
    t0: Option<std::time::Instant>,
    drag_start: (f64, f64),
}

fn now_iso() -> String {
    glib::DateTime::now_local()
        .and_then(|d| d.format("%Y-%m-%d %H:%M"))
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// The folder for the training files: «OnePepe entrenos» in your documents (Flatpak can write there).
fn default_folder() -> PathBuf {
    glib::user_special_dir(glib::UserDirectory::Documents)
        .unwrap_or_else(glib::home_dir)
        .join("OnePepe entrenos")
}

fn save(state: &mut State) -> anyhow::Result<()> {
    state.session.app_version = crate::globals::onepepe_version();
    let json = serde_json::to_string(&state.session)?;
    // write and rename: a crash never leaves half a file
    let tmp = state.path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &state.path)?;
    Ok(())
}

fn file_filter() -> gtk4::FileFilter {
    let filter = gtk4::FileFilter::new();
    filter.add_pattern(&format!("*{SUFFIX}"));
    filter.set_name(Some(&gettext("OnePepe training")));
    filter
}

/// «New training»: a new training file with the 200 expressions, then the training window.
pub(crate) fn new_training(appwindow: &RnAppWindow) {
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        async move {
            let Some(set) = choose_set(&appwindow).await else {
                return;
            };
            let request = serde_json::json!({"id": 0, "cmd": "training_new", "set": set});
            let Some(resp) = crate::mathhelper::ask(&appwindow, request.to_string()).await else {
                return;
            };
            if let Some(e) = crate::mathhelper::answer_error(&resp) {
                appwindow.overlays().dispatch_toast_error(&e);
                return;
            }
            let session = serde_json::from_str::<serde_json::Value>(&resp)
                .ok()
                .and_then(|v| serde_json::from_value::<Session>(v["session"].clone()).ok());
            let Some(session) = session else {
                appwindow.overlays().dispatch_toast_error(&gettext(
                    "The math helper could not create the training: update it (git pull)",
                ));
                return;
            };
            let folder = default_folder();
            let _ = std::fs::create_dir_all(&folder);
            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            let filter = file_filter();
            filters.append(&filter);
            let dialog = gtk4::FileDialog::builder()
                .title(gettext("New training"))
                .accept_label(gettext("Create"))
                .modal(true)
                .filters(&filters)
                .default_filter(&filter)
                .initial_folder(&gio::File::for_path(&folder))
                .initial_name(format!("{}{SUFFIX}", session.title.replace(':', "")))
                .build();
            let file = match dialog.save_future(Some(&appwindow)).await {
                Ok(file) => file,
                Err(e) => {
                    file_dialog_failed(&appwindow, &e);
                    return;
                }
            };
            let Some(mut path) = file.path() else {
                return;
            };
            if !path.to_string_lossy().ends_with(SUFFIX) {
                path = PathBuf::from(format!(
                    "{}{SUFFIX}",
                    path.to_string_lossy().trim_end_matches(".json")
                ));
            }
            let mut state = State {
                path,
                session,
                strokes: vec![],
                times: vec![],
                t0: None,
                drag_start: (0.0, 0.0),
            };
            if let Err(e) = save(&mut state) {
                appwindow.overlays().dispatch_toast_error(&format!(
                    "{}: {e}",
                    gettext("The training file could not be written")
                ));
                return;
            }
            crate::prefs::set(
                "ultimo-entreno",
                state.path.to_string_lossy().to_string().into(),
            );
            open_window(&appwindow, state);
        }
    ));
}

/// The file dialog failed: say why, unless the user just cancelled it.
fn file_dialog_failed(appwindow: &RnAppWindow, e: &glib::Error) {
    if e.matches(gtk4::DialogError::Dismissed) || e.matches(gtk4::DialogError::Cancelled) {
        return;
    }
    tracing::error!("The file dialog of the training failed, Err: {e:?}");
    appwindow
        .overlays()
        .dispatch_toast_error(&format!("{}: {e}", gettext("The file dialog failed")));
}

/// Asks which set of expressions to write (when there is more than one).
async fn choose_set(appwindow: &RnAppWindow) -> Option<String> {
    let resp = crate::mathhelper::ask(
        appwindow,
        r#"{"id": 0, "cmd": "training_sets"}"#.to_string(),
    )
    .await?;
    if let Some(e) = crate::mathhelper::answer_error(&resp) {
        appwindow.overlays().dispatch_toast_error(&e);
        return None;
    }
    let v = serde_json::from_str::<serde_json::Value>(&resp).ok()?;
    let sets = v["sets"].as_array()?.clone();
    match sets.len() {
        0 => None,
        1 => sets[0]["name"].as_str().map(String::from),
        _ => {
            let dialog = adw::AlertDialog::new(
                Some(&gettext("New training")),
                Some(&gettext("Which expressions do you want to write?")),
            );
            let body = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            for set in &sets {
                let label = gtk4::Label::new(Some(&format!(
                    "{} ({}): {}",
                    set["title"].as_str().unwrap_or_default(),
                    set["count"].as_u64().unwrap_or(0),
                    set["description"].as_str().unwrap_or_default()
                )));
                label.set_wrap(true);
                label.set_xalign(0.0);
                label.add_css_class("dim-label");
                body.append(&label);
            }
            dialog.set_extra_child(Some(&body));
            dialog.add_response("cancel", &gettext("Cancel"));
            for set in &sets {
                if let (Some(name), Some(title)) = (set["name"].as_str(), set["title"].as_str()) {
                    dialog.add_response(name, title);
                }
            }
            dialog.set_close_response("cancel");
            let answer = dialog.choose_future(Some(appwindow)).await;
            (answer.as_str() != "cancel").then(|| answer.to_string())
        }
    }
}

/// «Open training»: goes on with a training file.
pub(crate) fn open_training(appwindow: &RnAppWindow) {
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        async move {
            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            let filter = file_filter();
            filters.append(&filter);
            let builder = gtk4::FileDialog::builder()
                .title(gettext("Open training"))
                .modal(true)
                .filters(&filters)
                .default_filter(&filter);
            let last = crate::prefs::get_string("ultimo-entreno", "");
            let builder = if !last.is_empty() && Path::new(&last).exists() {
                builder.initial_file(&gio::File::for_path(&last))
            } else {
                builder.initial_folder(&gio::File::for_path(default_folder()))
            };
            let file = match builder.build().open_future(Some(&appwindow)).await {
                Ok(file) => file,
                Err(e) => {
                    file_dialog_failed(&appwindow, &e);
                    return;
                }
            };
            let Some(path) = file.path() else {
                return;
            };
            let session = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<Session>(&s).ok())
                .filter(|s| s.format == "onepepe-entreno");
            let Some(session) = session else {
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("This is not an OnePepe training file"));
                return;
            };
            crate::prefs::set("ultimo-entreno", path.to_string_lossy().to_string().into());
            open_window(
                &appwindow,
                State {
                    path,
                    session,
                    strokes: vec![],
                    times: vec![],
                    t0: None,
                    drag_start: (0.0, 0.0),
                },
            );
        }
    ));
}

/// Height of each guide line of the box, in pixels: like a line of a page, so you write at your usual size.
const LINE_HEIGHT: f64 = 130.0;

/// The widgets of the training window that change from one expression to the next. The window keeps them (in its
/// close handler) while it is open; everything else only has weak references, so nothing is kept after closing.
struct Widgets {
    counter: gtk4::Label,
    progress: gtk4::ProgressBar,
    hint: gtk4::Label,
    picture: gtk4::Picture,
    /// The strip of fixed height with the picture.
    strip: gtk4::ScrolledWindow,
    /// The expression as readable text, when it can't be drawn (matplotlib missing).
    plain: gtk4::Label,
    latex: gtk4::Label,
    area: gtk4::DrawingArea,
    message: gtk4::Label,
    previous: gtk4::Button,
    import: gtk4::Button,
}

fn open_window(appwindow: &RnAppWindow, state: State) {
    let state = Rc::new(RefCell::new(state));
    let window = adw::Window::builder()
        .title(gettext("Training"))
        .default_width(1100)
        .default_height(760)
        .transient_for(appwindow)
        .build();
    let header = adw::HeaderBar::new();
    let import = gtk4::Button::with_label(&gettext("Use for training"));
    import.add_css_class("suggested-action");
    import.set_tooltip_text(Some(&gettext(
        "Your handwriting goes to your examples (on this computer). Then you can retrain.",
    )));
    header.pack_end(&import);

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_bottom(16);
    vbox.set_margin_top(8);
    let top = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let counter = gtk4::Label::new(None);
    counter.add_css_class("heading");
    let progress = gtk4::ProgressBar::new();
    progress.set_hexpand(true);
    progress.set_valign(gtk4::Align::Center);
    top.append(&counter);
    top.append(&progress);
    let hint = gtk4::Label::new(None);
    hint.add_css_class("dim-label");
    hint.set_halign(gtk4::Align::Start);

    // the expression to write
    let picture = gtk4::Picture::new();
    picture.set_can_shrink(true);
    picture.set_content_fit(gtk4::ContentFit::ScaleDown);
    // in a strip of fixed height: a picture as wide as the window would grow as tall as the window
    let picture_strip = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::External)
        .min_content_height(120)
        .max_content_height(120)
        .propagate_natural_height(false)
        .child(&picture)
        .build();
    let plain = gtk4::Label::new(None);
    plain.add_css_class("title-1");
    plain.set_wrap(true);
    plain.set_justify(gtk4::Justification::Center);
    plain.set_visible(false);
    let latex = gtk4::Label::new(None);
    latex.set_selectable(true);
    latex.add_css_class("dim-label");
    latex.add_css_class("monospace");

    // the box to write in
    let area = gtk4::DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.add_css_class("view");
    area.add_css_class("onepepe-training-box");
    let frame = gtk4::Frame::new(None);
    frame.set_child(Some(&area));

    let message = gtk4::Label::new(None);
    message.add_css_class("dim-label");
    let buttons = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let previous = gtk4::Button::with_label(&gettext("← Previous"));
    let undo = gtk4::Button::with_label(&gettext("Undo stroke"));
    let clear = gtk4::Button::with_label(&gettext("Clear"));
    let skip = gtk4::Button::with_label(&gettext("Skip"));
    let next = gtk4::Button::with_label(&gettext("Next →"));
    next.add_css_class("suggested-action");
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    buttons.append(&previous);
    buttons.append(&undo);
    buttons.append(&clear);
    buttons.append(&spacer);
    buttons.append(&message);
    buttons.append(&skip);
    buttons.append(&next);

    vbox.append(&top);
    vbox.append(&hint);
    vbox.append(&picture_strip);
    vbox.append(&plain);
    vbox.append(&latex);
    vbox.append(&frame);
    vbox.append(&buttons);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&vbox));
    window.set_content(Some(&toolbar));

    let widgets = Rc::new(Widgets {
        counter,
        progress,
        hint,
        picture,
        strip: picture_strip.clone(),
        plain,
        latex,
        area: area.clone(),
        message,
        previous: previous.clone(),
        import: import.clone(),
    });

    // drawing: guide lines and your strokes
    area.set_draw_func(clone!(
        #[strong]
        state,
        move |area, cx, w, h| {
            let state = state.borrow();
            let n = state
                .session
                .items
                .get(state.session.current)
                .map(|i| i.lines.len().max(1))
                .unwrap_or(1);
            let color = area.color();
            let band = (h as f64 / n as f64).min(LINE_HEIGHT);
            // a guide line in each band, where the letters sit
            cx.set_source_rgba(
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                0.18,
            );
            cx.set_line_width(1.0);
            for i in 0..n {
                let y = band * (i as f64 + 0.7);
                cx.move_to(16.0, y);
                cx.line_to(w as f64 - 16.0, y);
                let _ = cx.stroke();
            }
            cx.set_source_rgba(
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                1.0,
            );
            cx.set_line_width(2.5);
            cx.set_line_cap(gtk4::cairo::LineCap::Round);
            cx.set_line_join(gtk4::cairo::LineJoin::Round);
            for stroke in &state.strokes {
                if let Some(first) = stroke.first() {
                    cx.move_to(first[0], first[1]);
                    if stroke.len() == 1 {
                        cx.line_to(first[0] + 0.1, first[1] + 0.1);
                    }
                    for p in &stroke[1..] {
                        cx.line_to(p[0], p[1]);
                    }
                    let _ = cx.stroke();
                }
            }
        }
    ));
    // writing with the pen (or the mouse)
    let drag = gtk4::GestureDrag::new();
    drag.set_button(0);
    drag.connect_drag_begin(clone!(
        #[strong]
        state,
        #[weak]
        widgets,
        move |_, x, y| {
            let mut s = state.borrow_mut();
            let t0 = *s.t0.get_or_insert_with(std::time::Instant::now);
            let t = t0.elapsed().as_secs_f64();
            s.drag_start = (x, y);
            s.strokes.push(vec![[x, y]]);
            s.times.push([t, t]);
            drop(s);
            widgets.message.set_text("");
            widgets.area.queue_draw();
        }
    ));
    drag.connect_drag_update(clone!(
        #[strong]
        state,
        #[weak]
        area,
        move |_, dx, dy| {
            let mut s = state.borrow_mut();
            let (x0, y0) = s.drag_start;
            let t = s.t0.map(|t0| t0.elapsed().as_secs_f64()).unwrap_or(0.0);
            if let Some(stroke) = s.strokes.last_mut() {
                stroke.push([x0 + dx, y0 + dy]);
            }
            if let Some(time) = s.times.last_mut() {
                time[1] = t;
            }
            drop(s);
            area.queue_draw();
        }
    ));
    area.add_controller(drag);

    undo.connect_clicked(clone!(
        #[strong]
        state,
        #[weak]
        area,
        move |_| {
            let mut s = state.borrow_mut();
            s.strokes.pop();
            s.times.pop();
            drop(s);
            area.queue_draw();
        }
    ));
    clear.connect_clicked(clone!(
        #[strong]
        state,
        #[weak]
        area,
        move |_| {
            let mut s = state.borrow_mut();
            s.strokes.clear();
            s.times.clear();
            s.t0 = None;
            drop(s);
            area.queue_draw();
        }
    ));
    next.connect_clicked(clone!(
        #[strong]
        state,
        #[weak]
        widgets,
        #[weak]
        appwindow,
        move |_| {
            if state.borrow().strokes.is_empty() {
                widgets
                    .message
                    .set_text(&gettext("Write it, or press Skip"));
                return;
            }
            keep_answer(&state, &widgets, false);
            go_to(&appwindow, &state, &widgets, 1);
        }
    ));
    skip.connect_clicked(clone!(
        #[strong]
        state,
        #[weak]
        widgets,
        #[weak]
        appwindow,
        move |_| {
            keep_answer(&state, &widgets, true);
            go_to(&appwindow, &state, &widgets, 1);
        }
    ));
    previous.connect_clicked(clone!(
        #[strong]
        state,
        #[weak]
        widgets,
        #[weak]
        appwindow,
        move |_| {
            if !state.borrow().strokes.is_empty() {
                keep_answer(&state, &widgets, false);
            }
            go_to(&appwindow, &state, &widgets, -1);
        }
    ));
    import.connect_clicked(clone!(
        #[strong]
        state,
        #[weak]
        widgets,
        #[weak]
        appwindow,
        move |_| {
            if !state.borrow().strokes.is_empty() {
                keep_answer(&state, &widgets, false);
            }
            import_for_training(&appwindow, &state, &widgets);
        }
    ));
    // what is written in the box is kept when the window is closed
    window.connect_close_request(clone!(
        #[strong]
        state,
        #[strong]
        widgets,
        move |_| {
            if !state.borrow().strokes.is_empty() {
                keep_answer(&state, &widgets, false);
            }
            glib::Propagation::Proceed
        }
    ));

    show_item(appwindow, &state, &widgets);
    window.present();
}

/// Keeps what is written in the box as the answer of the current expression and saves the file.
fn keep_answer(state: &Rc<RefCell<State>>, widgets: &Widgets, skipped: bool) {
    let mut s = state.borrow_mut();
    let Some(item) = s.session.items.get(s.session.current).cloned() else {
        return;
    };
    let size = [widgets.area.width() as f64, widgets.area.height() as f64];
    let answer = Answer {
        strokes: if skipped { vec![] } else { s.strokes.clone() },
        times: if skipped { vec![] } else { s.times.clone() },
        size,
        line_height: LINE_HEIGHT.min(size[1] / item.lines.len().max(1) as f64),
        lines: item.lines.len(),
        written: now_iso(),
        skipped,
        imported: false,
    };
    s.session.answers.insert(item.id, answer);
    if let Err(e) = save(&mut s) {
        tracing::error!("Saving the training file failed, Err: {e:?}");
        widgets
            .message
            .set_text(&gettext("The training file could not be written"));
    }
}

/// Goes to the next (+1) or previous (-1) expression.
fn go_to(appwindow: &RnAppWindow, state: &Rc<RefCell<State>>, widgets: &Rc<Widgets>, step: i32) {
    {
        let mut s = state.borrow_mut();
        let n = s.session.items.len();
        let current = s.session.current as i64 + step as i64;
        s.session.current = current.clamp(0, n as i64) as usize;
        let _ = save(&mut s);
    }
    show_item(appwindow, state, widgets);
}

/// Shows the current expression (and what was already written for it, when going back).
fn show_item(appwindow: &RnAppWindow, state: &Rc<RefCell<State>>, widgets: &Rc<Widgets>) {
    let (item, total, written, pending, answer) = {
        let mut s = state.borrow_mut();
        let item = s.session.items.get(s.session.current).cloned();
        let answer = item
            .as_ref()
            .and_then(|i| s.session.answers.get(&i.id))
            .cloned()
            .unwrap_or_default();
        s.strokes = answer.strokes.clone();
        s.times = answer.times.clone();
        s.t0 = None;
        (
            item,
            s.session.items.len(),
            s.session.written(),
            s.session.to_be_imported(),
            answer,
        )
    };
    let current = state.borrow().session.current;
    widgets
        .counter
        .set_text(&format!("{} / {total}", (current + 1).min(total)));
    widgets
        .progress
        .set_fraction(written as f64 / total.max(1) as f64);
    widgets.previous.set_sensitive(current > 0);
    widgets.import.set_sensitive(pending > 0);
    widgets.message.set_text("");
    let Some(item) = item else {
        // the end
        widgets.hint.set_text(&gettext(
            "Done! Press «Use for training», and then «Retrain» in the Math tab.",
        ));
        widgets.picture.set_paintable(None::<&gdk::Paintable>);
        widgets.latex.set_text("");
        widgets.area.queue_draw();
        return;
    };
    let mut hint = match item.kind.as_str() {
        "symbols" => gettext("The same symbol six times, with a gap between them"),
        "steps" => gettext("One step on each line, as in class"),
        _ => gettext("Write it as you would in class"),
    };
    // the subject, when the set says it (it is written in Spanish in the set)
    if let Some(subject) = item.hint.as_ref().filter(|_| item.kind == "expr") {
        hint = format!("{subject} · {hint}");
    }
    if answer.skipped {
        hint = format!("{hint} · {}", gettext("skipped"));
    } else if answer.imported {
        hint = format!("{hint} · {}", gettext("already used for training"));
    }
    widgets.hint.set_text(&hint);
    widgets.latex.set_text(&item.lines.join("   ⏎   "));
    widgets.picture.set_paintable(None::<&gdk::Paintable>);
    widgets.area.queue_draw();
    if let Some(window) = widgets.area.root().and_downcast::<gtk4::Window>() {
        window.set_title(Some(&format!(
            "{} — {}",
            gettext("Training"),
            state.borrow().session.title
        )));
    }
    // the expression drawn by the helper (the lines of the steps one under the other)
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        #[weak]
        widgets,
        #[strong]
        state,
        async move {
            // in the color of the text, so it is seen in the light and the dark theme
            let c = widgets.latex.color();
            let color = format!(
                "#{:02x}{:02x}{:02x}",
                (c.red() * 255.0) as u8,
                (c.green() * 255.0) as u8,
                (c.blue() * 255.0) as u8
            );
            let request = serde_json::json!({
                "id": 0,
                "cmd": "training_render",
                "lines": item.lines,
                "color": color,
            });
            let Some(resp) = crate::mathhelper::ask(&appwindow, request.to_string()).await else {
                return;
            };
            // the user may have gone on meanwhile
            if state
                .borrow()
                .session
                .items
                .get(state.borrow().session.current)
                .map(|i| &i.id)
                != Some(&item.id)
            {
                return;
            }
            let v = serde_json::from_str::<serde_json::Value>(&resp).unwrap_or_default();
            let texture = v["png"]
                .as_str()
                .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
                .and_then(|png| gdk::Texture::from_bytes(&glib::Bytes::from(&png)).ok());
            match texture {
                Some(texture) => {
                    widgets.picture.set_paintable(Some(&texture));
                    widgets.plain.set_visible(false);
                    widgets.strip.set_visible(true);
                }
                None => {
                    // not drawn: the expression as text, big, and how to get it drawn
                    widgets.strip.set_visible(false);
                    widgets
                        .plain
                        .set_text(v["text"].as_str().unwrap_or(&item.lines.join("\n")));
                    widgets.plain.set_visible(true);
                    if v["missing"].as_str() == Some("matplotlib") {
                        widgets.message.set_text(&gettext(
                            "To see it drawn, install matplotlib once: cd tools/onepepe-math && ./instalar.sh",
                        ));
                    }
                }
            }
        }
    ));
}

/// «Use for training»: the math helper turns what you wrote into examples of your handwriting.
fn import_for_training(appwindow: &RnAppWindow, state: &Rc<RefCell<State>>, widgets: &Rc<Widgets>) {
    let session = match serde_json::to_value(&state.borrow().session) {
        Ok(v) => v,
        Err(_) => return,
    };
    widgets.import.set_sensitive(false);
    widgets
        .message
        .set_text(&gettext("Learning your handwriting…"));
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        #[weak]
        widgets,
        #[strong]
        state,
        async move {
            let request =
                serde_json::json!({"id": 0, "cmd": "training_import", "session": session});
            let resp = crate::mathhelper::ask(&appwindow, request.to_string()).await;
            let v = resp
                .and_then(|r| serde_json::from_str::<serde_json::Value>(&r).ok())
                .unwrap_or_default();
            if let Some(e) = v["error"].as_str() {
                widgets.message.set_text(e);
                widgets.import.set_sensitive(true);
                return;
            }
            let imported = v["imported"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let failed = v["failed"].as_array().map(|a| a.len()).unwrap_or(0);
            let symbols = v["symbols"].as_u64().unwrap_or(0);
            {
                let mut s = state.borrow_mut();
                for id in &imported {
                    if let Some(a) = s.session.answers.get_mut(id) {
                        a.imported = true;
                    }
                }
                let _ = save(&mut s);
            }
            widgets.message.set_text("");
            let body = gettext(
                "{n} expressions ({symbols} symbols of your handwriting) are now among your examples. {failed} did not fit (more or fewer strokes than symbols) and were left out. Retrain to use them in the model; the corrections count at once.",
            )
            .replace("{n}", &imported.len().to_string())
            .replace("{symbols}", &symbols.to_string())
            .replace("{failed}", &failed.to_string());
            let dialog = adw::AlertDialog::new(Some(&gettext("Handwriting saved")), Some(&body));
            dialog.add_response("close", &gettext("Close"));
            dialog.add_response("retrain", &gettext("Retrain now"));
            dialog.set_response_appearance("retrain", adw::ResponseAppearance::Suggested);
            let parent = widgets.area.root().and_downcast::<gtk4::Window>();
            if dialog.choose_future(parent.as_ref()).await.as_str() == "retrain" {
                crate::mathhelper::retrain(&appwindow);
            }
            show_item(&appwindow, &state, &widgets);
        }
    ));
}
