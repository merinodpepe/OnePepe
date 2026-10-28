// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: small preferences of the app that do not need a GSettings key (they live in
// ~/.config/onepepe/ajustes.json, never in the .rnote files).

// Imports
use gtk4::glib;
use serde_json::{Map, Value};
use std::cell::RefCell;
use tracing::error;

thread_local! {
    static PREFS: RefCell<Option<Map<String, Value>>> = const { RefCell::new(None) };
}

fn path() -> std::path::PathBuf {
    glib::user_config_dir().join("onepepe").join("ajustes.json")
}

fn with<R>(f: impl FnOnce(&mut Map<String, Value>) -> R) -> R {
    PREFS.with_borrow_mut(|prefs| {
        let prefs = prefs.get_or_insert_with(|| {
            std::fs::read_to_string(path())
                .ok()
                .and_then(|s| serde_json::from_str::<Map<String, Value>>(&s).ok())
                .unwrap_or_default()
        });
        f(prefs)
    })
}

pub(crate) fn get(key: &str) -> Option<Value> {
    with(|prefs| prefs.get(key).cloned())
}

pub(crate) fn get_bool(key: &str, default: bool) -> bool {
    get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

pub(crate) fn get_string(key: &str, default: &str) -> String {
    get(key)
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| default.to_string())
}

pub(crate) fn set(key: &str, value: Value) {
    let json = with(|prefs| {
        prefs.insert(key.to_string(), value);
        serde_json::to_string_pretty(prefs)
    });
    let result = json.map_err(anyhow::Error::from).and_then(|json| {
        std::fs::create_dir_all(path().parent().unwrap())?;
        std::fs::write(path(), json)?;
        Ok(())
    });
    if let Err(e) = result {
        error!("Could not save the OnePepe preferences, Err: {e:?}");
    }
}
