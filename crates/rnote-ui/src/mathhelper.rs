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

thread_local! {
    static HELPER: RefCell<Option<Rc<MathHelper>>> = const { RefCell::new(None) };
    static SPAWN_FAILED: Cell<bool> = const { Cell::new(false) };
    static TIMER: RefCell<Option<glib::SourceId>> = const { RefCell::new(None) };
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

async fn recognize_now(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let Some(request) = canvas.engine_mut().math_take_request(notebook_dir(canvas)) else {
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
        }
    }
}

/// How a LaTeX label is shown on the candidate buttons.
fn display_label(latex: &str) -> String {
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
                let requests = canvas.engine_mut().math_correction_requests(
                    symbol_id,
                    &latex,
                    group_check.is_active(),
                    notebook_dir(&canvas),
                );
                popover.popdown();
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
                                        canvas.engine_mut().math_apply_correction(&resp);
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
        ));
        flow.append(&button);
    }
    vbox.append(&flow);
    vbox.append(&group_check);
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
