// OnePepe: create a new page (an empty .rnote file) in the current folder of the workspace browser and open it.

// Imports
use crate::workspacebrowser::widgethelper;
use crate::{RnAppWindow, RnWorkspaceBrowser};
use gettextrs::gettext;
use gtk4::{Align, Entry, Label, gio, glib, glib::clone, pango, prelude::*};
use rnote_engine::Engine;
use std::path::{Path, PathBuf};
use tracing::{debug, error, warn};

/// Create a new `create-page` action.
pub(crate) fn create_page(
    workspacebrowser: &RnWorkspaceBrowser,
    appwindow: &RnAppWindow,
) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("create-page", None);

    action.connect_activate(clone!(
        #[weak]
        workspacebrowser,
        #[weak]
        appwindow,
        move |_, _| {
            let Some(parent_path) = workspacebrowser.dir_list_file().and_then(|f| f.path()) else {
                warn!("Can't create new page when there currently is no workspace selected");
                return;
            };
            let name_entry = Entry::builder()
                .placeholder_text(gettext("Page Name"))
                .text(next_free_page_name(&parent_path))
                .build();
            let title_label = Label::builder()
                .margin_bottom(12)
                .halign(Align::Center)
                .label(gettext("New Page"))
                .width_chars(24)
                .ellipsize(pango::EllipsizeMode::End)
                .build();
            title_label.add_css_class("title-4");
            let (apply_button, popover) =
                widgethelper::create_entry_dialog(&name_entry, &title_label);
            workspacebrowser.dir_controls_actions_box().append(&popover);

            name_entry.connect_changed(clone!(
                #[weak]
                apply_button,
                #[strong]
                parent_path,
                move |entry| {
                    let valid =
                        page_path(&parent_path, &entry.text()).is_some_and(|path| !path.exists());
                    apply_button.set_sensitive(valid);
                    if valid {
                        entry.remove_css_class("error");
                    } else {
                        entry.add_css_class("error");
                    }
                }
            ));

            apply_button.connect_clicked(clone!(
                #[weak]
                popover,
                #[weak]
                name_entry,
                #[weak]
                appwindow,
                move |_| {
                    popover.popdown();
                    let Some(path) = page_path(&parent_path, &name_entry.text()) else {
                        return;
                    };
                    if path.exists() {
                        appwindow
                            .overlays()
                            .dispatch_toast_error(&gettext("A page with this name already exists"));
                        return;
                    }
                    glib::spawn_future_local(clone!(
                        #[weak]
                        appwindow,
                        async move {
                            if let Err(e) = write_empty_page(&appwindow, &path).await {
                                error!("Creating new page `{}` failed, Err: {e:?}", path.display());
                                appwindow
                                    .overlays()
                                    .dispatch_toast_error(&gettext("Creating new page failed"));
                                return;
                            }
                            appwindow
                                .open_file_w_dialogs(gio::File::for_path(&path), None, true)
                                .await;
                        }
                    ));
                }
            ));

            popover.popup();
            name_entry.grab_focus();
            name_entry.select_region(0, -1);
        }
    ));

    action
}

/// The path of the new page, adding the `.rnote` extension. None if the name is not usable.
fn page_path(parent: &Path, name: &str) -> Option<PathBuf> {
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.starts_with('.') {
        return None;
    }
    let file_name = if name.ends_with(".rnote") {
        name.to_string()
    } else {
        format!("{name}.rnote")
    };
    Some(parent.join(file_name))
}

/// "Page 1", "Page 2", ... the first one that does not exist yet.
fn next_free_page_name(parent: &Path) -> String {
    let base = gettext("Page");
    (1..)
        .map(|i| format!("{base} {i}"))
        .find(|name| page_path(parent, name).is_some_and(|p| !p.exists()))
        .unwrap()
}

/// Writes an empty document with the current document preset (format, background, layout).
pub(crate) async fn write_empty_page(appwindow: &RnAppWindow, path: &Path) -> anyhow::Result<()> {
    let mut engine = Engine::default();
    engine.document.config = appwindow.document_config_preset_ref().clone();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let bytes = engine.save_as_rnote_bytes(file_name).await??;
    debug!("Writing new empty page to `{}`", path.display());
    async_fs::write(path, bytes).await?;
    Ok(())
}
