// OnePepe: move a page or section (file or folder) to another folder, for example to another section of the notebook.

// Imports
use crate::RnAppWindow;
use crate::workspacebrowser::RnFileRow;
use gettextrs::gettext;
use gtk4::{FileDialog, gio, glib, glib::clone, prelude::*};
use std::path::PathBuf;
use tracing::{debug, error};

/// Create a new `move-to` action.
pub(crate) fn move_to(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("move-to", None);
    action.connect_activate(clone!(
        #[weak]
        filerow,
        #[weak]
        appwindow,
        move |_, _| {
            let Some(current_path) = filerow.current_file().and_then(|f| f.path()) else {
                return;
            };
            let Some(file_name) = current_path.file_name().map(|n| n.to_os_string()) else {
                return;
            };
            // start in the notebook (the selected workspace)
            let workspace_dir = appwindow
                .sidebar()
                .workspacebrowser()
                .workspacesbar()
                .selected_workspacelistentry()
                .map(|e| PathBuf::from(e.dir()));
            let mut builder = FileDialog::builder()
                .title(gettext("Move to Folder"))
                .accept_label(gettext("Move Here"))
                .modal(true);
            if let Some(dir) = workspace_dir.as_ref() {
                builder = builder.initial_folder(&gio::File::for_path(dir));
            }
            let dialog = builder.build();

            glib::spawn_future_local(clone!(
                #[weak]
                appwindow,
                async move {
                    let target_dir = match dialog.select_folder_future(Some(&appwindow)).await {
                        Ok(f) => match f.path() {
                            Some(p) => p,
                            None => return,
                        },
                        Err(e) => {
                            debug!("Did not move file (dialog dismissed or error), Err: {e:?}");
                            return;
                        }
                    };
                    let target = target_dir.join(&file_name);
                    if target == current_path {
                        return;
                    }
                    if target.starts_with(&current_path) {
                        appwindow
                            .overlays()
                            .dispatch_toast_error(&gettext("Can't move a folder into itself"));
                        return;
                    }
                    if target.exists() {
                        appwindow.overlays().dispatch_toast_error(&gettext(
                            "Moving failed, the target folder already has an item with this name",
                        ));
                        return;
                    }
                    appwindow.overlays().progressbar_start_pulsing();
                    if let Err(e) = async_fs::rename(&current_path, &target).await {
                        error!(
                            "Moving `{}` to `{}` failed, Err: {e:?}",
                            current_path.display(),
                            target.display()
                        );
                        appwindow
                            .overlays()
                            .dispatch_toast_error(&gettext("Moving failed"));
                        appwindow.overlays().progressbar_abort();
                    } else {
                        crate::mathhelper::sidecar_follow_move(&current_path, &target).await;
                        appwindow.overlays().progressbar_finish();
                    }
                }
            ));
        }
    ));
    action
}
