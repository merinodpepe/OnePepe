// OnePepe: communication with the external math helper program `onepepe-math` (see tools/onepepe-math).
//
// The helper is started once, when it is needed for the first time, and reads/writes one JSON message per line.

// Imports
use crate::{RnAppWindow, RnCanvas};
use anyhow::Context;
use futures::lock::Mutex;
use gettextrs::gettext;
use gtk4::{gdk, gio, glib, glib::clone, prelude::*};
use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::rc::Rc;
use std::time::Duration;

/// Pause after the last stroke before the strokes are recognized.
const RECOGNITION_DELAY: Duration = Duration::from_millis(800);

pub(crate) struct MathHelper {
    _process: gio::Subprocess,
    stdin: gio::OutputStream,
    stdout: gio::DataInputStream,
    lock: Mutex<()>,
}

/// Pause after the last change before the page (results, variables, steps) is asked again.
const PAGE_DELAY: Duration = Duration::from_millis(400);

thread_local! {
    static HELPER: RefCell<Option<Rc<MathHelper>>> = const { RefCell::new(None) };
    static SPAWN_FAILED: Cell<bool> = const { Cell::new(false) };
    static TIMER: RefCell<Option<glib::SourceId>> = const { RefCell::new(None) };
    static PAGE_TIMER: RefCell<Option<glib::SourceId>> = const { RefCell::new(None) };
    /// The dictionary of symbols (`names` of the helper), asked once: (latex, character, group, first name).
    static NAMES: RefCell<Option<Rc<Vec<NameEntry>>>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone)]
struct NameEntry {
    latex: String,
    char: String,
    group: String,
    name: String,
    shortcut: String,
}

/// The command that starts the helper: `ONEPEPE_MATH_CMD`, or `onepepe-math` (through `flatpak-spawn --host` inside
/// Flatpak, where Python and its libraries are on the host).
fn helper_command() -> Vec<String> {
    if let Ok(cmd) = std::env::var("ONEPEPE_MATH_CMD") {
        let parts = cmd.split_whitespace().map(String::from).collect::<Vec<_>>();
        if !parts.is_empty() {
            return parts;
        }
    }
    if std::env::var_os("FLATPAK_ID").is_some() {
        vec![
            "flatpak-spawn".into(),
            "--host".into(),
            "onepepe-math".into(),
        ]
    } else {
        vec!["onepepe-math".into()]
    }
}

impl MathHelper {
    fn spawn() -> anyhow::Result<Self> {
        let argv = helper_command();
        let argv_os = argv.iter().map(OsStr::new).collect::<Vec<&OsStr>>();
        let process = gio::Subprocess::newv(
            &argv_os,
            gio::SubprocessFlags::STDIN_PIPE | gio::SubprocessFlags::STDOUT_PIPE,
        )?;
        let stdin = process.stdin_pipe().context("no stdin")?;
        let stdout = gio::DataInputStream::new(&process.stdout_pipe().context("no stdout")?);
        Ok(Self {
            _process: process,
            stdin,
            stdout,
            lock: Mutex::new(()),
        })
    }

    /// Sends one request line and waits for the response line.
    pub(crate) async fn request(&self, line: String) -> anyhow::Result<String> {
        let _guard = self.lock.lock().await;
        let mut bytes = line.into_bytes();
        bytes.push(b'\n');
        self.stdin
            .write_all_future(bytes, glib::Priority::DEFAULT)
            .await
            .map_err(|(_, e)| e)?;
        self.stdin.flush_future(glib::Priority::DEFAULT).await?;
        let resp = self
            .stdout
            .read_line_utf8_future(glib::Priority::DEFAULT)
            .await?;
        resp.map(|s| s.to_string())
            .context("the math helper closed its output")
    }
}

/// The running helper, started if needed. Shows an error once if it can't be started.
fn helper(appwindow: &RnAppWindow) -> Option<Rc<MathHelper>> {
    if let Some(h) = HELPER.with(|h| h.borrow().clone()) {
        return Some(h);
    }
    match MathHelper::spawn() {
        Ok(h) => {
            let h = Rc::new(h);
            HELPER.with(|cell| *cell.borrow_mut() = Some(h.clone()));
            Some(h)
        }
        Err(e) => {
            tracing::error!("Starting the math helper failed, Err: {e:?}");
            if !SPAWN_FAILED.with(|f| f.replace(true)) {
                appwindow.overlays().dispatch_toast_error(&gettext(
                    "Could not start the math helper \"onepepe-math\". See tools/onepepe-math/README.md",
                ));
            }
            None
        }
    }
}

/// The helper broke (e.g. it was closed): start a new one next time.
fn reset_helper() {
    HELPER.with(|h| *h.borrow_mut() = None);
}

fn notebook_dir(canvas: &RnCanvas) -> Option<String> {
    canvas
        .output_file()
        .and_then(|f| f.parent())
        .and_then(|p| p.path())
        .map(|p| p.to_string_lossy().to_string())
}

/// Recognizes the queued strokes after a short pause without writing.
pub(crate) fn schedule_recognition(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    TIMER.with(|t| {
        if let Some(id) = t.borrow_mut().take() {
            id.remove();
        }
    });
    let id = glib::timeout_add_local_once(
        RECOGNITION_DELAY,
        clone!(
            #[weak]
            appwindow,
            #[weak]
            canvas,
            move || {
                TIMER.with(|t| t.borrow_mut().take());
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    #[weak]
                    canvas,
                    async move {
                        recognize_now(&appwindow, &canvas).await;
                    }
                ));
            }
        ),
    );
    TIMER.with(|t| *t.borrow_mut() = Some(id));
}

/// Recognizes what is waiting: the strokes just written and the lines queued with «Recognize selection».
async fn recognize_now(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    loop {
        // (the engine is borrowed only for this statement)
        let request = canvas.engine_mut().math_take_request(notebook_dir(canvas));
        let Some(request) = request else {
            return;
        };
        let Some(helper) = helper(appwindow) else {
            return;
        };
        match helper.request(request).await {
            Ok(resp) => {
                let widget_flags = canvas.engine_mut().math_apply_response(&resp);
                appwindow.handle_widget_flags(widget_flags, canvas);
            }
            Err(e) => {
                tracing::error!("Math recognition failed, Err: {e:?}");
                reset_helper();
                return;
            }
        }
    }
}

/// One request to the helper and its answer, or None (with a message if it can't be started).
pub(crate) async fn ask(appwindow: &RnAppWindow, request: String) -> Option<String> {
    let helper = helper(appwindow)?;
    match helper.request(request).await {
        Ok(resp) => Some(resp),
        Err(e) => {
            tracing::error!("Asking the math helper failed, Err: {e:?}");
            reset_helper();
            None
        }
    }
}

/// Asks again, after a short pause, what goes next to the recognized lines (results, "?", variables, wrong steps).
pub(crate) fn schedule_page(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    PAGE_TIMER.with(|t| {
        if let Some(id) = t.borrow_mut().take() {
            id.remove();
        }
    });
    let id = glib::timeout_add_local_once(
        PAGE_DELAY,
        clone!(
            #[weak]
            appwindow,
            #[weak]
            canvas,
            move || {
                PAGE_TIMER.with(|t| t.borrow_mut().take());
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    #[weak]
                    canvas,
                    async move {
                        page_now(&appwindow, &canvas).await;
                    }
                ));
            }
        ),
    );
    PAGE_TIMER.with(|t| *t.borrow_mut() = Some(id));
}

async fn page_now(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let Some(request) = canvas.engine_mut().math_page_request(notebook_dir(canvas)) else {
        // nothing is written next to the lines any more
        canvas.queue_draw();
        return;
    };
    let Some(helper) = helper(appwindow) else {
        return;
    };
    match helper.request(request).await {
        Ok(resp) => {
            let widget_flags = canvas.engine_mut().math_apply_page(&resp);
            appwindow.handle_widget_flags(widget_flags, canvas);
        }
        Err(e) => {
            tracing::error!("Asking the math helper for the page failed, Err: {e:?}");
            reset_helper();
        }
    }
}

/// «Recognize selection»: recognizes the selected handwriting (e.g. written before math mode was on).
pub(crate) fn recognize_selection(appwindow: &RnAppWindow) {
    let Some(canvas) = appwindow.active_tab_canvas() else {
        return;
    };
    let lines = canvas.engine_mut().math_queue_selection();
    if lines == 0 {
        appwindow.overlays().dispatch_toast_text(
            &gettext("Select handwriting with the lasso first"),
            crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
        );
        return;
    }
    // math mode on, so the border and the taps on the symbols work
    if !canvas.engine_ref().math_mode() {
        let _ = gtk4::prelude::WidgetExt::activate_action(appwindow, "win.math-mode", None);
    }
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        #[weak]
        canvas,
        async move {
            recognize_now(&appwindow, &canvas).await;
        }
    ));
}

/// «Copy as LaTeX»: the selected recognized lines go to the clipboard as LaTeX, one line each.
pub(crate) fn copy_latex(appwindow: &RnAppWindow) {
    let Some(canvas) = appwindow.active_tab_canvas() else {
        return;
    };
    let Some(request) = canvas.engine_mut().math_latex_request() else {
        appwindow.overlays().dispatch_toast_text(
            &gettext("Select recognized math with the lasso first"),
            crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
        );
        return;
    };
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        async move {
            let Some(helper) = helper(&appwindow) else {
                return;
            };
            match helper.request(request).await {
                Ok(resp) => {
                    if let Some(latex) = rnote_engine::Engine::math_latex_from_response(&resp) {
                        appwindow.clipboard().set_text(&latex);
                        appwindow.overlays().dispatch_toast_text(
                            &gettext("LaTeX copied"),
                            crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Asking the math helper for LaTeX failed, Err: {e:?}");
                    reset_helper();
                }
            }
        }
    ));
}

/// «Plot»: draws the functions of the selected lines (y = f(x)) next to them, as normal strokes.
pub(crate) fn plot_selection(appwindow: &RnAppWindow) {
    let Some(canvas) = appwindow.active_tab_canvas() else {
        return;
    };
    let Some(request) = canvas.engine_mut().math_plot_request() else {
        appwindow.overlays().dispatch_toast_text(
            &gettext("Select a recognized function with the lasso first, like y = x²"),
            crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
        );
        return;
    };
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        #[weak]
        canvas,
        async move {
            let Some(helper) = helper(&appwindow) else {
                return;
            };
            match helper.request(request).await {
                Ok(resp) => {
                    let result = canvas.engine_mut().math_apply_plot(&resp);
                    match result {
                        Ok(widget_flags) => appwindow.handle_widget_flags(widget_flags, &canvas),
                        Err(e) => {
                            appwindow.overlays().dispatch_toast_text(
                                &format!("{}: {e}", gettext("Nothing to plot")),
                                crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Asking the math helper for a plot failed, Err: {e:?}");
                    reset_helper();
                }
            }
        }
    ));
}

/// The dictionary of symbols, asked to the helper the first time.
async fn names(appwindow: &RnAppWindow) -> Option<Rc<Vec<NameEntry>>> {
    if let Some(names) = NAMES.with(|n| n.borrow().clone()) {
        return Some(names);
    }
    let helper = helper(appwindow)?;
    let resp = helper
        .request(r#"{"id": 0, "cmd": "names"}"#.to_string())
        .await
        .ok()?;
    let entries = parse_names(&resp);
    if entries.is_empty() {
        return None;
    }
    let entries = Rc::new(entries);
    NAMES.with(|n| *n.borrow_mut() = Some(entries.clone()));
    Some(entries)
}

fn parse_names(resp: &str) -> Vec<NameEntry> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(resp) else {
        return vec![];
    };
    v["entries"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|e| NameEntry {
                    latex: e["latex"].as_str().unwrap_or_default().to_string(),
                    char: e["char"].as_str().unwrap_or_default().to_string(),
                    group: e["group"].as_str().unwrap_or_default().to_string(),
                    name: e["names"][0].as_str().unwrap_or_default().to_string(),
                    shortcut: e["shortcut"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Corrects the symbol (or its whole group) to `latex`: the helper learns it and draws it clean.
fn correct_symbol(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    symbol_id: u64,
    latex: &str,
    whole_group: bool,
) {
    let requests = canvas.engine_mut().math_correction_requests(
        symbol_id,
        latex,
        whole_group,
        notebook_dir(canvas),
    );
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        #[weak]
        canvas,
        async move {
            let Some(helper) = helper(&appwindow) else {
                return;
            };
            for request in requests {
                match helper.request(request).await {
                    Ok(resp) => {
                        let widget_flags = canvas.engine_mut().math_apply_correction(&resp);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                        note_correction(&appwindow);
                    }
                    Err(e) => {
                        tracing::error!("Math correction failed, Err: {e:?}");
                        reset_helper();
                        return;
                    }
                }
            }
        }
    ));
}

/// «Other…»: write which symbol it is when it is not among the candidates. By name in Spanish or English («delta»,
/// «menor o igual»), the character («δ»), its LaTeX («\delta») or a short cut («\d», «<=»), with suggestions while
/// typing; or picked from a grid of the hard to type symbols.
fn other_symbol_box(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    popover: &gtk4::Popover,
    group_check: &gtk4::CheckButton,
    symbol_id: u64,
) -> gtk4::Box {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    let entry = gtk4::SearchEntry::builder()
        .placeholder_text(gettext("Other… (delta, \\d, δ, <=)"))
        .build();
    let suggestions = gtk4::FlowBox::builder()
        .max_children_per_line(4)
        .selection_mode(gtk4::SelectionMode::None)
        .build();
    let grid_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    let expander = gtk4::Expander::new(Some(&gettext("All symbols")));
    expander.set_child(Some(&grid_box));
    vbox.append(&entry);
    vbox.append(&suggestions);
    vbox.append(&expander);

    let choose = Rc::new(clone!(
        #[weak]
        appwindow,
        #[weak]
        canvas,
        #[weak]
        popover,
        #[weak]
        group_check,
        move |latex: String| {
            popover.popdown();
            correct_symbol(
                &appwindow,
                &canvas,
                symbol_id,
                &latex,
                group_check.is_active(),
            );
        }
    ));
    let symbol_button = {
        let choose = choose.clone();
        move |e: &NameEntry, with_name: bool| {
            let label = if with_name {
                format!("{}  {}", e.char, e.name)
            } else {
                e.char.clone()
            };
            let button = gtk4::Button::with_label(&label);
            button.add_css_class("flat");
            let tooltip = if e.shortcut.is_empty() {
                format!("{}  ·  {}", e.name, e.latex)
            } else {
                format!("{}  ·  {}  ·  {}", e.name, e.latex, e.shortcut)
            };
            button.set_tooltip_text(Some(&tooltip));
            let latex = e.latex.clone();
            let choose = choose.clone();
            button.connect_clicked(move |_| choose(latex.clone()));
            button
        }
    };
    // the grid, by groups, when the dictionary arrives
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        #[weak]
        grid_box,
        #[strong]
        symbol_button,
        async move {
            let Some(names) = names(&appwindow).await else {
                return;
            };
            let mut groups: Vec<&str> = vec![];
            for e in names.iter() {
                if !groups.contains(&e.group.as_str()) {
                    groups.push(&e.group);
                }
            }
            for group in groups {
                let title = gtk4::Label::new(Some(group));
                title.add_css_class("caption");
                title.add_css_class("dim-label");
                title.set_halign(gtk4::Align::Start);
                grid_box.append(&title);
                let flow = gtk4::FlowBox::builder()
                    .max_children_per_line(10)
                    .selection_mode(gtk4::SelectionMode::None)
                    .build();
                for e in names.iter().filter(|e| e.group == group) {
                    flow.append(&symbol_button(e, false));
                }
                grid_box.append(&flow);
            }
        }
    ));
    // suggestions while typing
    let exact: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    entry.connect_search_changed(clone!(
        #[weak]
        appwindow,
        #[weak]
        suggestions,
        #[strong]
        exact,
        #[strong]
        symbol_button,
        move |entry| {
            let query = entry.text().to_string();
            glib::spawn_future_local(clone!(
                #[weak]
                appwindow,
                #[weak]
                suggestions,
                #[strong]
                exact,
                #[strong]
                symbol_button,
                async move {
                    while let Some(child) = suggestions.first_child() {
                        suggestions.remove(&child);
                    }
                    *exact.borrow_mut() = None;
                    if query.trim().is_empty() {
                        return;
                    }
                    let Some(helper) = helper(&appwindow) else {
                        return;
                    };
                    let request = serde_json::json!({"id": 0, "cmd": "names", "query": query});
                    let Ok(resp) = helper.request(request.to_string()).await else {
                        reset_helper();
                        return;
                    };
                    let v = serde_json::from_str::<serde_json::Value>(&resp).unwrap_or_default();
                    *exact.borrow_mut() = v["exact"].as_str().map(String::from);
                    for e in parse_names(&resp).iter().take(8) {
                        suggestions.append(&symbol_button(e, true));
                    }
                }
            ));
        }
    ));
    // Enter: what was typed if it is clear, or the first suggestion
    entry.connect_activate(clone!(
        #[weak]
        suggestions,
        #[strong]
        exact,
        #[strong]
        choose,
        move |_| {
            if let Some(latex) = exact.borrow().clone() {
                choose(latex);
            } else if let Some(button) = suggestions
                .child_at_index(0)
                .and_then(|c| c.child())
                .and_downcast::<gtk4::Button>()
            {
                button.emit_clicked();
            }
        }
    ));
    vbox
}

/// A tap on what is written next to a line: the "?" says what was read, a wrong step its explanation.
fn explain_overlay(
    canvas: &RnCanvas,
    pos: rnote_engine::engine::math::Vector2,
    kind: &str,
    text: &str,
    latex: &str,
) {
    let message = match kind {
        "unknown" => gettext(
            "It can't be calculated. I read: {latex}\nTap the symbol that was read wrongly to correct it.",
        )
        .replace("{latex}", latex),
        "error" => format!(
            "{}\n{}",
            text.trim_start_matches("✗ "),
            gettext("The step is compared with the one before, with random values.")
        ),
        _ => format!("{text}\nLaTeX: {latex}"),
    };
    let surface_pos = canvas.engine_ref().camera.transform().transform_point2(pos);
    let popover = gtk4::Popover::new();
    let label = gtk4::Label::new(Some(&message));
    label.set_wrap(true);
    label.set_max_width_chars(40);
    label.set_selectable(true);
    popover.set_child(Some(&label));
    popover.set_parent(canvas);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(
        surface_pos.x as i32,
        surface_pos.y as i32,
        1,
        1,
    )));
    popover.connect_closed(|popover| {
        let popover = popover.clone();
        glib::idle_add_local_once(move || popover.unparent());
    });
    popover.popup();
}

/// How a LaTeX label is shown on the candidate buttons.
fn display_label(latex: &str) -> String {
    if let Some(e) = NAMES.with(|n| {
        n.borrow()
            .as_ref()
            .and_then(|names| names.iter().find(|e| e.latex == latex).cloned())
    }) {
        return e.char;
    }
    let table: &[(&str, &str)] = &[
        (r"\alpha", "α"),
        (r"\beta", "β"),
        (r"\gamma", "γ"),
        (r"\delta", "δ"),
        (r"\epsilon", "ϵ"),
        (r"\varepsilon", "ε"),
        (r"\zeta", "ζ"),
        (r"\eta", "η"),
        (r"\theta", "θ"),
        (r"\vartheta", "ϑ"),
        (r"\iota", "ι"),
        (r"\kappa", "κ"),
        (r"\lambda", "λ"),
        (r"\mu", "μ"),
        (r"\nu", "ν"),
        (r"\xi", "ξ"),
        (r"\pi", "π"),
        (r"\rho", "ρ"),
        (r"\sigma", "σ"),
        (r"\tau", "τ"),
        (r"\upsilon", "υ"),
        (r"\phi", "ϕ"),
        (r"\varphi", "φ"),
        (r"\chi", "χ"),
        (r"\psi", "ψ"),
        (r"\omega", "ω"),
        (r"\Gamma", "Γ"),
        (r"\Delta", "Δ"),
        (r"\Theta", "Θ"),
        (r"\Lambda", "Λ"),
        (r"\Xi", "Ξ"),
        (r"\Pi", "Π"),
        (r"\Sigma", "Σ"),
        (r"\Phi", "Φ"),
        (r"\Psi", "Ψ"),
        (r"\Omega", "Ω"),
        (r"\times", "×"),
        (r"\cdot", "·"),
        (r"\div", "÷"),
        (r"\pm", "±"),
        (r"\mp", "∓"),
        (r"\neq", "≠"),
        (r"\leq", "≤"),
        (r"\geq", "≥"),
        (r"\approx", "≈"),
        (r"\sim", "∼"),
        (r"\equiv", "≡"),
        (r"\propto", "∝"),
        (r"\infty", "∞"),
        (r"\partial", "∂"),
        (r"\nabla", "∇"),
        (r"\int", "∫"),
        (r"\sum", "∑"),
        (r"\prod", "∏"),
        (r"\sqrt", "√"),
        (r"\rightarrow", "→"),
        (r"\leftarrow", "←"),
        (r"\in", "∈"),
        (r"\frac", "—"),
        (r"\{", "{"),
        (r"\}", "}"),
        (r"\circ", "°"),
    ];
    table
        .iter()
        .find(|(l, _)| *l == latex)
        .map(|(_, u)| u.to_string())
        .unwrap_or_else(|| latex.to_string())
}

/// Shows the candidates of the tapped symbol. Choosing one corrects the symbol, or its whole group.
pub(crate) fn show_candidates(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let Some(pos) = canvas.engine_ref().math_last_tap() else {
        return;
    };
    if let Some((kind, text, latex)) = canvas.engine_ref().math_overlay_at(pos) {
        explain_overlay(canvas, pos, &kind, &text, &latex);
        return;
    }
    let Some((symbol_id, candidates)) = canvas.engine_ref().math_symbol_at(pos) else {
        return;
    };
    let surface_pos = canvas.engine_ref().camera.transform().transform_point2(pos);

    let popover = gtk4::Popover::new();
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    let flow = gtk4::FlowBox::builder()
        .max_children_per_line(6)
        .selection_mode(gtk4::SelectionMode::None)
        .build();
    let group_check = gtk4::CheckButton::with_label(&gettext("Apply to the whole group"));
    for (latex, p) in candidates.iter().take(8) {
        let button = gtk4::Button::with_label(&display_label(latex));
        button.set_tooltip_text(Some(&format!("{latex}  ({:.0} %)", p * 100.0)));
        let latex = latex.clone();
        button.connect_clicked(clone!(
            #[weak]
            appwindow,
            #[weak]
            canvas,
            #[weak]
            popover,
            #[weak]
            group_check,
            move |_| {
                popover.popdown();
                correct_symbol(
                    &appwindow,
                    &canvas,
                    symbol_id,
                    &latex,
                    group_check.is_active(),
                );
            }
        ));
        flow.append(&button);
    }
    vbox.append(&flow);
    vbox.append(&group_check);
    vbox.append(&other_symbol_box(
        appwindow,
        canvas,
        &popover,
        &group_check,
        symbol_id,
    ));
    // fix the cut into symbols: two strokes that belong together, or one symbol that is really two
    let cut_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    for (label, join) in [
        (gettext("Join with the one next to it"), true),
        (gettext("Split into strokes"), false),
    ] {
        let button = gtk4::Button::with_label(&label);
        button.add_css_class("flat");
        button.connect_clicked(clone!(
            #[weak]
            appwindow,
            #[weak]
            canvas,
            #[weak]
            popover,
            move |_| {
                let requests = if join {
                    canvas
                        .engine_mut()
                        .math_join_with_next_requests(symbol_id, notebook_dir(&canvas))
                } else {
                    canvas
                        .engine_mut()
                        .math_split_requests(symbol_id, notebook_dir(&canvas))
                };
                popover.popdown();
                if requests.is_empty() {
                    appwindow.overlays().dispatch_toast_text(
                        &gettext("Nothing to join or split here"),
                        crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                    );
                    return;
                }
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    #[weak]
                    canvas,
                    async move {
                        let Some(helper) = helper(&appwindow) else {
                            return;
                        };
                        for request in requests {
                            match helper.request(request).await {
                                Ok(resp) => {
                                    let widget_flags =
                                        canvas.engine_mut().math_apply_group_fix(&resp);
                                    appwindow.handle_widget_flags(widget_flags, &canvas);
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Fixing the cut into symbols failed, Err: {e:?}"
                                    );
                                    reset_helper();
                                    return;
                                }
                            }
                        }
                        note_correction(&appwindow);
                    }
                ));
            }
        ));
        cut_box.append(&button);
    }
    vbox.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    vbox.append(&cut_box);
    popover.set_child(Some(&vbox));
    popover.set_parent(canvas);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(
        surface_pos.x as i32,
        surface_pos.y as i32,
        1,
        1,
    )));
    popover.connect_closed(|popover| {
        let popover = popover.clone();
        // unparent after the closing animation
        glib::idle_add_local_once(move || popover.unparent());
    });
    popover.popup();
}

// ---------- OnePepe: hidden file next to the page with the recognized math symbols ----------

/// Writes (or removes, when there is nothing to remember) the hidden math file of a page that was just saved.
pub(crate) async fn save_sidecar(canvas: &RnCanvas, page: &std::path::Path) {
    let path = rnote_engine::engine::math::math_sidecar_path(page);
    let json = canvas.engine_ref().math_sidecar_json();
    let res = match json {
        Some(json) => async_fs::write(&path, json).await,
        None if path.exists() => async_fs::remove_file(&path).await,
        None => Ok(()),
    };
    if let Err(e) = res {
        tracing::warn!(
            "Writing the hidden math file `{}` failed, Err: {e:?}",
            path.display()
        );
    }
}

/// Restores the recognized math symbols of a page that was just opened.
pub(crate) async fn load_sidecar(canvas: &RnCanvas, page: &std::path::Path) {
    let path = rnote_engine::engine::math::math_sidecar_path(page);
    if let Ok(json) = async_fs::read_to_string(&path).await {
        let n = canvas.engine_mut().math_load_sidecar(&json);
        tracing::debug!(
            "Restored {n} recognized math symbols from `{}`",
            path.display()
        );
    }
}

/// The hidden math file follows its page when the page is renamed or moved.
pub(crate) async fn sidecar_follow_move(from: &std::path::Path, to: &std::path::Path) {
    let source = rnote_engine::engine::math::math_sidecar_path(from);
    if source.exists()
        && let Err(e) =
            async_fs::rename(&source, rnote_engine::engine::math::math_sidecar_path(to)).await
    {
        tracing::warn!("Moving the hidden math file failed, Err: {e:?}");
    }
}

/// The hidden math file is copied when the page is duplicated.
pub(crate) async fn sidecar_follow_copy(from: &std::path::Path, to: &std::path::Path) {
    let source = rnote_engine::engine::math::math_sidecar_path(from);
    if source.exists()
        && let Err(e) =
            async_fs::copy(&source, rnote_engine::engine::math::math_sidecar_path(to)).await
    {
        tracing::warn!("Copying the hidden math file failed, Err: {e:?}");
    }
}

/// The hidden math file goes to the trash with its page.
pub(crate) fn sidecar_follow_trash(page: &std::path::Path) {
    let path = rnote_engine::engine::math::math_sidecar_path(page);
    if path.exists()
        && let Err(e) = gio::File::for_path(&path).trash(None::<&gio::Cancellable>)
    {
        tracing::warn!("Trashing the hidden math file failed, Err: {e:?}");
    }
}

// ---------- «Reentrenar»: retrain the symbol recognizer with your handwriting ----------

thread_local! {
    static RETRAINING: Cell<bool> = const { Cell::new(false) };
    static CORRECTIONS_SINCE_CHECK: Cell<u32> = const { Cell::new(0) };
    static SUGGESTED: Cell<bool> = const { Cell::new(false) };
}

/// Suggest retraining once this many of your examples are new since the last retraining.
const SUGGEST_AFTER_NEW_EXAMPLES: u64 = 50;

/// Runs `onepepe-math <arg>` and returns its standard output.
async fn run_helper_once(arg: &str) -> anyhow::Result<String> {
    let mut argv = helper_command();
    argv.push(arg.to_string());
    let argv_os = argv.iter().map(OsStr::new).collect::<Vec<&OsStr>>();
    let process = gio::Subprocess::newv(&argv_os, gio::SubprocessFlags::STDOUT_PIPE)?;
    let (stdout, _) = process.communicate_utf8_future(None).await?;
    Ok(stdout.map(|s| s.to_string()).unwrap_or_default())
}

async fn retrain_estimate() -> anyhow::Result<serde_json::Value> {
    let out = run_helper_once("--estimar").await?;
    Ok(serde_json::from_str(out.trim())?)
}

/// The «Reentrenar» button: says how many examples there are and how long it takes, asks, and retrains in the
/// background with the progress in a notification. The app can be used meanwhile.
pub(crate) fn retrain(appwindow: &RnAppWindow) {
    use adw::prelude::*;
    if RETRAINING.with(|r| r.get()) {
        appwindow.overlays().dispatch_toast_text(
            &gettext("Retraining is already running"),
            crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
        );
        return;
    }
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        async move {
            let estimate = match retrain_estimate().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        "Asking the math helper for the retraining estimate failed, Err: {e:?}"
                    );
                    appwindow.overlays().dispatch_toast_error(&gettext(
                        "Could not start the math helper \"onepepe-math\". See tools/onepepe-math/README.md",
                    ));
                    return;
                }
            };
            if !estimate["dataset"].as_bool().unwrap_or(false) {
                appwindow.overlays().dispatch_toast_error(&gettext(
                    "Retraining needs the MathWriting data: run tools/onepepe-math/instalar.sh --entrenar once",
                ));
                return;
            }
            let examples = estimate["examples"].as_u64().unwrap_or(0);
            let new = estimate["new_since_last"].as_u64().unwrap_or(0);
            let minutes = estimate["minutes"].as_f64().unwrap_or(5.0).max(1.0).round();
            let body = gettext(
                "You have {examples} examples of your handwriting ({new} new since the last retraining). It takes about {minutes} minutes and you can keep using the app meanwhile. The new model is only used if it is not worse with your recent examples.",
            )
            .replace("{examples}", &examples.to_string())
            .replace("{new}", &new.to_string())
            .replace("{minutes}", &minutes.to_string());
            let dialog =
                adw::AlertDialog::new(Some(&gettext("Retrain with my handwriting")), Some(&body));
            dialog.add_response("cancel", &gettext("Cancel"));
            dialog.add_response("retrain", &gettext("Retrain now"));
            dialog.set_response_appearance("retrain", adw::ResponseAppearance::Suggested);
            dialog.set_default_response(Some("retrain"));
            dialog.set_close_response("cancel");
            if dialog.choose_future(Some(&appwindow)).await.as_str() == "retrain" {
                run_retraining(&appwindow).await;
            }
        }
    ));
}

async fn run_retraining(appwindow: &RnAppWindow) {
    let mut argv = helper_command();
    argv.push("--reentrenar".to_string());
    let argv_os = argv.iter().map(OsStr::new).collect::<Vec<&OsStr>>();
    let process = match gio::Subprocess::newv(&argv_os, gio::SubprocessFlags::STDOUT_PIPE) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Starting the retraining failed, Err: {e:?}");
            appwindow
                .overlays()
                .dispatch_toast_error(&gettext("Retraining could not be started"));
            return;
        }
    };
    RETRAINING.with(|r| r.set(true));
    let toast_ref = appwindow.overlays().dispatch_toast_w_button(
        &gettext("Retraining with your handwriting…"),
        &gettext("Cancel"),
        clone!(
            #[weak]
            process,
            // SIGTERM (not a kill): flatpak-spawn passes it on to the retraining, which stops the training too
            move |_| process.send_signal(15)
        ),
        None,
    );
    let Some(stdout) = process.stdout_pipe() else {
        RETRAINING.with(|r| r.set(false));
        return;
    };
    let stdout = gio::DataInputStream::new(&stdout);
    let mut last: Option<serde_json::Value> = None;
    while let Ok(Some(line)) = stdout.read_line_utf8_future(glib::Priority::DEFAULT).await {
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if let (Some(toast), Some(text)) = (toast_ref.upgrade(), msg["message"].as_str()) {
            let pct = (msg["progress"].as_f64().unwrap_or(0.0) * 100.0).round();
            toast.set_title(&format!("{} — {} ({pct} %)", gettext("Retraining"), text));
        }
        last = Some(msg);
    }
    let _ = process.wait_future().await;
    RETRAINING.with(|r| r.set(false));
    if let Some(toast) = toast_ref.upgrade() {
        toast.dismiss();
    }
    match last {
        Some(msg) if msg["stage"] == "done" => {
            if msg["replaced"].as_bool().unwrap_or(false) {
                // the helper loads the new model when it starts again
                reset_helper();
            }
            appwindow.overlays().dispatch_toast_text(
                msg["message"].as_str().unwrap_or_default(),
                Some(Duration::from_secs(10)),
            );
        }
        Some(msg) if msg["stage"] == "error" => {
            appwindow
                .overlays()
                .dispatch_toast_error(msg["message"].as_str().unwrap_or_default());
        }
        _ => {
            appwindow.overlays().dispatch_toast_text(
                &gettext("Retraining was cancelled; the current model is kept"),
                crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
            );
        }
    }
}

/// After corrections: every 10, checks how many examples are new and suggests retraining (once per session) when
/// there are enough. Never starts on its own.
fn note_correction(appwindow: &RnAppWindow) {
    let n = CORRECTIONS_SINCE_CHECK.with(|c| {
        c.set(c.get() + 1);
        c.get()
    });
    if n % 10 != 0 || SUGGESTED.with(|s| s.get()) || RETRAINING.with(|r| r.get()) {
        return;
    }
    glib::spawn_future_local(clone!(
        #[weak]
        appwindow,
        async move {
            let Ok(estimate) = retrain_estimate().await else {
                return;
            };
            if estimate["new_since_last"].as_u64().unwrap_or(0) >= SUGGEST_AFTER_NEW_EXAMPLES
                && estimate["dataset"].as_bool().unwrap_or(false)
                && !SUGGESTED.with(|s| s.replace(true))
            {
                appwindow.overlays().dispatch_toast_w_button(
                    &gettext("You have many new corrections: retraining would learn your handwriting better"),
                    &gettext("Retrain"),
                    clone!(
                        #[weak]
                        appwindow,
                        move |_| retrain(&appwindow)
                    ),
                    Some(Duration::from_secs(15)),
                );
            }
        }
    ));
}
