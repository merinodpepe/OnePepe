// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: crash report. If the app closes because of an error (a Rust panic), what happened is written to
// `fallos.log` in the app's data folder (with Flatpak: ~/.var/app/<id>/data/onepepe/fallos.log), with the version,
// the message, where in the code and the backtrace. The next start says where the file is, so it can be sent.

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::glib;
use std::io::Write;
use std::path::PathBuf;

/// Where the crash reports are written.
pub(crate) fn log_path() -> PathBuf {
    glib::user_data_dir().join("onepepe").join("fallos.log")
}

/// Installs the hook that writes the report. The usual message on the terminal is kept.
pub(crate) fn install() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let path = log_path();
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(&path));
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let now = glib::DateTime::now_local()
                .and_then(|d| d.format("%Y-%m-%d %H:%M:%S"))
                .map(|s| s.to_string())
                .unwrap_or_default();
            let _ = writeln!(
                f,
                "=== {now} · OnePepe {} ===\n{info}\n{backtrace}\n",
                crate::globals::onepepe_version()
            );
        }
        default_hook(info);
    }));
}

/// At start: if there is a crash report newer than the last one seen, says where it is.
pub(crate) fn notify_previous(appwindow: &RnAppWindow) {
    let path = log_path();
    let Ok(modified) = std::fs::metadata(&path).and_then(|m| m.modified()) else {
        return;
    };
    let stamp = modified
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let seen = crate::prefs::get("fallo-visto")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if stamp <= seen {
        return;
    }
    crate::prefs::set("fallo-visto", stamp.into());
    appwindow.overlays().dispatch_toast_text(
        &gettext("OnePepe closed because of an error last time. The report is in {path}: send it to fix it")
            .replace("{path}", &path.to_string_lossy()),
        Some(std::time::Duration::from_secs(30)),
    );
}
