// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the tree of the notebook, like the page list of OneNote: sections (folders) → pages → subpages, with
// 32 px rows and 16 px indentation. Drag a page onto another to put it before it (at its level), onto a section to
// move it there. The order and the subpages are kept in a hidden index file of each folder (onepepe_index.rs), so
// the files never move or get renamed for it. Also: a search field, «+ Page», the open rows remembered, and the
// panel folded to the notebooks bar.

// Imports
use super::RnWorkspaceBrowser;
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gdk, gio, glib, glib::clone, prelude::*, subclass::prelude::*};
use rnote_engine::onepepe_index::{Entry, FolderIndex};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;
use tracing::error;

/// The state of the tree.
#[derive(Default)]
struct Tree {
    root: RefCell<Option<PathBuf>>,
    model: RefCell<Option<gtk4::TreeListModel>>,
    selection: RefCell<Option<gtk4::SingleSelection>>,
    monitors: RefCell<Vec<gio::FileMonitor>>,
    rebuild_pending: Cell<bool>,
    restoring: Cell<bool>,
    search: RefCell<String>,
}

thread_local! {
    static TREE: Rc<Tree> = Rc::new(Tree::default());
}

fn tree() -> Rc<Tree> {
    TREE.with(|t| t.clone())
}

fn info_path(info: &gio::FileInfo) -> Option<PathBuf> {
    info.attribute_object("standard::file")
        .and_downcast::<gio::File>()
        .and_then(|f| f.path())
}

fn item_path(item: Option<glib::Object>) -> Option<PathBuf> {
    let item = item?;
    let info = match item.downcast::<gtk4::TreeListRow>() {
        Ok(row) => row.item()?.downcast::<gio::FileInfo>().ok()?,
        Err(item) => item.downcast::<gio::FileInfo>().ok()?,
    };
    info_path(&info)
}

fn file_info(path: &Path, is_dir: bool) -> gio::FileInfo {
    let info = gio::FileInfo::new();
    info.set_attribute_object("standard::file", &gio::File::for_path(path));
    info.set_file_type(if is_dir {
        gio::FileType::Directory
    } else {
        gio::FileType::Regular
    });
    let icon = if is_dir {
        "workspacelistentryicon-folder-symbolic"
    } else if path.extension().is_some_and(|e| e == "rnote") {
        "workspacelistentryicon-document-symbolic"
    } else {
        "text-x-generic-symbolic"
    };
    info.set_icon(&gio::ThemedIcon::new(icon));
    info
}

/// The rows of one level: the top of a folder (`parent` None) or the subpages of a page.
fn level(dir: &Path, parent: Option<&str>) -> gio::ListStore {
    let store = gio::ListStore::new::<gio::FileInfo>();
    let entries = FolderIndex::read_entries(dir);
    for entry in FolderIndex::load(dir).children(&entries, parent) {
        store.append(&file_info(&dir.join(&entry.name), entry.is_dir));
    }
    store
}

fn children_of(item: &glib::Object) -> Option<gio::ListModel> {
    let info = item.downcast_ref::<gio::FileInfo>()?;
    let path = info_path(info)?;
    if info.file_type() == gio::FileType::Directory {
        return Some(level(&path, None).upcast());
    }
    let name = path.file_name()?.to_string_lossy().to_string();
    if !name.ends_with(".rnote") {
        return None;
    }
    let store = level(path.parent()?, Some(&name));
    (store.n_items() > 0).then(|| store.upcast())
}

fn expanded_paths() -> BTreeSet<String> {
    crate::prefs::get("arbol-abiertos")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Builds the tree of the notebook folder again (keeps the open rows).
fn rebuild(browser: &RnWorkspaceBrowser) {
    let tree = tree();
    tree.rebuild_pending.set(false);
    let Some(root) = browser.dir_list_dir() else {
        return;
    };
    tree.root.replace(Some(root.clone()));
    let search = tree.search.borrow().to_lowercase();
    let root_model: gio::ListModel = if search.is_empty() {
        level(&root, None).upcast()
    } else {
        search_results(&root, &search).upcast()
    };
    let searching = !search.is_empty();
    let model = gtk4::TreeListModel::new(root_model, false, false, move |item| {
        if searching { None } else { children_of(item) }
    });
    // restore the open rows
    tree.restoring.set(true);
    let open = expanded_paths();
    let mut i = 0;
    while i < model.n_items() {
        if let Some(row) = model.row(i)
            && let Some(path) = item_path(row.item())
            && open.contains(&path.to_string_lossy().to_string())
        {
            row.set_expanded(true);
        }
        i += 1;
    }
    tree.restoring.set(false);
    model.connect_items_changed(|model, _, _, _| remember_expanded(model));

    let selection = gtk4::SingleSelection::builder()
        .model(&model)
        .autoselect(false)
        .can_unselect(true)
        .build();
    selection.set_selected(gtk4::INVALID_LIST_POSITION);
    browser.imp().files_listview.set_model(Some(&selection));
    tree.model.replace(Some(model));
    tree.selection.replace(Some(selection));
    watch_folders(browser, &root, &open);
}

/// Saves which rows are open (the sections and pages expanded).
fn remember_expanded(model: &gtk4::TreeListModel) {
    let tree = tree();
    if tree.restoring.get() || !tree.search.borrow().is_empty() {
        return;
    }
    let mut open = BTreeSet::new();
    for i in 0..model.n_items() {
        if let Some(row) = model.row(i)
            && row.is_expanded()
            && let Some(path) = item_path(row.item())
        {
            open.insert(path.to_string_lossy().to_string());
        }
    }
    if open != expanded_paths() {
        crate::prefs::set("arbol-abiertos", serde_json::json!(open));
    }
}

/// Follows the changes of the folders shown (new pages, renames, other programs).
fn watch_folders(browser: &RnWorkspaceBrowser, root: &Path, open: &BTreeSet<String>) {
    let tree = tree();
    let mut monitors = vec![];
    let dirs = std::iter::once(root.to_path_buf()).chain(
        open.iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir() && p.starts_with(root)),
    );
    for dir in dirs {
        match gio::File::for_path(&dir)
            .monitor_directory(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
        {
            Ok(monitor) => {
                monitor.connect_changed(clone!(
                    #[weak]
                    browser,
                    move |_, _, _, _| schedule_rebuild(&browser)
                ));
                monitors.push(monitor);
            }
            Err(e) => error!("Could not watch the folder {dir:?}, Err: {e:?}"),
        }
    }
    tree.monitors.replace(monitors);
}

pub(crate) fn schedule_rebuild(browser: &RnWorkspaceBrowser) {
    let tree = tree();
    if tree.rebuild_pending.replace(true) {
        return;
    }
    glib::timeout_add_local_once(
        Duration::from_millis(250),
        clone!(
            #[weak]
            browser,
            move || rebuild(&browser)
        ),
    );
}

/// The pages and sections whose name contains the search, in all the notebook (a flat list).
fn search_results(root: &Path, search: &str) -> gio::ListStore {
    let store = gio::ListStore::new::<gio::FileInfo>();
    let mut dirs = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = dirs.pop() {
        for entry in FolderIndex::read_entries(&dir) {
            let path = dir.join(&entry.name);
            if display_name(&entry.name).to_lowercase().contains(search) && store.n_items() < 500 {
                store.append(&file_info(&path, entry.is_dir));
            }
            if entry.is_dir && depth < 8 {
                dirs.push((path, depth + 1));
            }
        }
    }
    store
}

/// Pages are shown by their name, like in OneNote.
fn display_name(name: &str) -> String {
    name.strip_suffix(".rnote").unwrap_or(name).to_string()
}

/// Changes the index of a folder and shows the result.
fn edit_index(
    browser: &RnWorkspaceBrowser,
    dir: &Path,
    f: impl FnOnce(&mut FolderIndex, &[Entry]) -> bool,
) {
    let mut index = FolderIndex::load(dir);
    let entries = FolderIndex::read_entries(dir);
    if f(&mut index, &entries) {
        if let Err(e) = index.save(dir) {
            error!("Could not save the order of the folder {dir:?}, Err: {e:?}");
        }
        schedule_rebuild(browser);
    }
}

/// A page or section dropped on a row: before a page (at its level), or into a section.
fn drop_on(browser: &RnWorkspaceBrowser, dragged: &gio::File, target: &Path) -> bool {
    let Some(source) = dragged.path() else {
        return false;
    };
    if source == target || target.starts_with(&source) {
        return false;
    }
    let Some(name) = source.file_name().map(|n| n.to_string_lossy().to_string()) else {
        return false;
    };
    let (dir, before) = if target.is_dir() {
        (target.to_path_buf(), None)
    } else {
        let Some(dir) = target.parent() else {
            return false;
        };
        (
            dir.to_path_buf(),
            target.file_name().map(|n| n.to_string_lossy().to_string()),
        )
    };
    if source.parent() != Some(dir.as_path()) {
        // into another folder: move the file (never over another one)
        let destination = dir.join(&name);
        if destination.exists() {
            return false;
        }
        if let Err(e) = std::fs::rename(&source, &destination) {
            error!("Could not move {source:?} to {destination:?}, Err: {e:?}");
            return false;
        }
    }
    edit_index(browser, &dir, |index, entries| {
        let parent = before
            .as_deref()
            .and_then(|b| index.parent_of(b, entries))
            .map(str::to_string);
        index.set_parent(&name, parent.as_deref());
        index.move_before(entries, &name, before.as_deref());
        true
    });
    true
}

pub(crate) fn setup(browser: &RnWorkspaceBrowser, appwindow: &RnAppWindow) {
    let imp = browser.imp();
    let listview = imp.files_listview.get();
    listview.add_css_class("onepepe-tree");
    listview.set_factory(Some(&row_factory(browser, appwindow)));
    listview.set_header_factory(gtk4::ListItemFactory::NONE);
    listview.set_single_click_activate(false);
    listview.connect_activate(clone!(
        #[weak]
        appwindow,
        move |listview, position| {
            let Some(item) = listview.model().and_then(|m| m.item(position)) else {
                return;
            };
            let Ok(row) = item.downcast::<gtk4::TreeListRow>() else {
                return;
            };
            let Some(path) = item_path(Some(row.clone().upcast())) else {
                return;
            };
            if path.is_dir() {
                row.set_expanded(!row.is_expanded());
                return;
            }
            glib::spawn_future_local(clone!(
                #[weak]
                appwindow,
                async move {
                    appwindow
                        .open_file_w_dialogs(gio::File::for_path(path), None, true)
                        .await;
                }
            ));
        }
    ));

    // search and «+ Page» above the tree
    let tools = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    tools.set_margin_start(6);
    tools.set_margin_end(6);
    tools.set_margin_top(6);
    let search = gtk4::SearchEntry::builder()
        .placeholder_text(gettext("Search pages"))
        .build();
    search.connect_search_changed(clone!(
        #[weak]
        browser,
        move |search| {
            tree().search.replace(search.text().to_string());
            rebuild(&browser);
        }
    ));
    let add_page = gtk4::Button::builder()
        .child(&{
            let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            content.append(&gtk4::Image::from_icon_name("list-add-symbolic"));
            content.append(&gtk4::Label::new(Some(&gettext("Page"))));
            content
        })
        .tooltip_text(gettext(
            "New page in the notebook (or in the section of the open page)",
        ))
        .halign(gtk4::Align::Start)
        .build();
    add_page.add_css_class("flat");
    add_page.add_css_class("onepepe-add-page");
    add_page.connect_clicked(clone!(
        #[weak]
        appwindow,
        #[weak]
        browser,
        move |_| {
            glib::spawn_future_local(clone!(
                #[weak]
                appwindow,
                #[weak]
                browser,
                async move { new_page(&appwindow, &browser).await }
            ));
        }
    ));
    tools.append(&search);
    tools.append(&add_page);
    imp.dir_box.append(&tools);

    // the panel folded to the notebooks bar (48 px) or open (260 px)
    let fold = gtk4::ToggleButton::builder()
        .icon_name("pan-start-symbolic")
        .tooltip_text(gettext("Fold the panel"))
        .valign(gtk4::Align::Center)
        .active(crate::prefs::get_bool("panel-plegado", false))
        .build();
    fold.add_css_class("flat");
    imp.dir_controls_actions_box.prepend(&fold);
    let unfold = gtk4::Button::builder()
        .icon_name("pan-end-symbolic")
        .tooltip_text(gettext("Unfold the panel"))
        .build();
    unfold.add_css_class("flat");
    unfold.set_visible(false);
    unfold.connect_clicked(clone!(
        #[weak]
        fold,
        move |_| fold.set_active(false)
    ));
    // with the buttons of the notebooks bar
    match imp
        .workspacesbar
        .imp()
        .move_selected_workspace_up_button
        .parent()
        .and_downcast::<gtk4::Box>()
    {
        Some(buttons) => buttons.prepend(&unfold),
        None => error!("Could not place the button to unfold the panel"),
    }
    // the settings page of the panel must not keep the panel wide
    appwindow.sidebar().sidebar_stack().set_hhomogeneous(false);
    let apply_widths = clone!(
        #[weak]
        appwindow,
        #[weak]
        browser,
        #[weak]
        fold,
        #[weak]
        unfold,
        move || {
            let folded = fold.is_active();
            let on_tree = appwindow.sidebar().sidebar_stack().visible_child()
                == Some(browser.clone().upcast());
            browser.imp().dir_box.set_visible(!folded);
            browser.imp().files_scroller.set_visible(!folded);
            unfold.set_visible(folded);
            // folded, only the notebooks bar stays: the switcher of the panel would keep it wide
            let sidebar = appwindow.sidebar();
            sidebar.headerbar().set_show_title(!(folded && on_tree));
            sidebar.appmenu().set_visible(!(folded && on_tree));
            let split = appwindow.split_view();
            let (min, max, fraction) = match (on_tree, folded) {
                (true, true) => (48.0, 64.0, 0.05),
                (true, false) => (260.0, 320.0, 0.2),
                // the settings need more room
                (false, _) => (500.0, 550.0, 0.3),
            };
            split.set_min_sidebar_width(min);
            split.set_max_sidebar_width(max);
            split.set_sidebar_width_fraction(fraction);
        }
    );
    let apply_widths = Rc::new(apply_widths);
    fold.connect_toggled(clone!(
        #[strong]
        apply_widths,
        move |fold| {
            crate::prefs::set("panel-plegado", fold.is_active().into());
            apply_widths();
        }
    ));
    appwindow
        .sidebar()
        .sidebar_stack()
        .connect_visible_child_notify(clone!(
            #[strong]
            apply_widths,
            move |_| apply_widths()
        ));
    apply_widths();

    // the tree follows the notebook folder
    imp.dir_list.connect_file_notify(clone!(
        #[weak]
        browser,
        move |_| schedule_rebuild(&browser)
    ));
    schedule_rebuild(browser);

    // the row of the open page is marked (brand-tint and a brand bar)
    glib::timeout_add_local(
        Duration::from_millis(500),
        clone!(
            #[weak]
            appwindow,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                mark_open_page(&appwindow);
                glib::ControlFlow::Continue
            }
        ),
    );
}

/// The section of the open page (its folder, inside the notebook), or the notebook folder.
pub(crate) fn current_section(appwindow: &RnAppWindow) -> Option<PathBuf> {
    let root = appwindow.sidebar().workspacebrowser().dir_list_dir()?;
    let open_dir = appwindow
        .active_tab_canvas()
        .and_then(|c| c.output_file())
        .and_then(|f| f.path())
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .filter(|dir| dir.starts_with(&root));
    Some(open_dir.unwrap_or(root))
}

/// «+ Page»: a new page in the section of the open page, right after it (at its level), and opens it.
async fn new_page(appwindow: &RnAppWindow, browser: &RnWorkspaceBrowser) {
    let Some(dir) = current_section(appwindow) else {
        return;
    };
    let open_name = appwindow
        .active_tab_canvas()
        .and_then(|c| c.output_file())
        .and_then(|f| f.path())
        .filter(|p| p.parent() == Some(dir.as_path()))
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
    let base = gettext("Untitled page");
    let path = (1..1000)
        .map(|n| {
            if n == 1 {
                dir.join(format!("{base}.rnote"))
            } else {
                dir.join(format!("{base} {n}.rnote"))
            }
        })
        .find(|p| !p.exists())
        .unwrap_or_else(|| dir.join(format!("{base}.rnote")));
    if let Err(e) = super::workspaceactions::createpage::write_empty_page(appwindow, &path).await {
        error!("Creating new page `{}` failed, Err: {e:?}", path.display());
        appwindow
            .overlays()
            .dispatch_toast_error(&gettext("Creating new page failed"));
        return;
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    if let Some(open_name) = open_name {
        edit_index(browser, &dir, |index, entries| {
            let parent = index.parent_of(&open_name, entries).map(str::to_string);
            index.set_parent(&name, parent.as_deref());
            // right after the open page
            let after = entries
                .iter()
                .skip_while(|e| e.name != open_name)
                .nth(1)
                .map(|e| e.name.clone())
                .filter(|n| *n != name);
            index.move_before(entries, &name, after.as_deref());
            true
        });
    }
    appwindow
        .open_file_w_dialogs(gio::File::for_path(&path), None, true)
        .await;
}

fn mark_open_page(appwindow: &RnAppWindow) {
    let tree = tree();
    let (Some(model), Some(selection)) =
        (tree.model.borrow().clone(), tree.selection.borrow().clone())
    else {
        return;
    };
    let open = appwindow
        .active_tab_canvas()
        .and_then(|c| c.output_file())
        .and_then(|f| f.path());
    let position = open.and_then(|open| {
        (0..model.n_items())
            .find(|i| model.row(*i).and_then(|r| item_path(r.item())) == Some(open.clone()))
    });
    let position = position.unwrap_or(gtk4::INVALID_LIST_POSITION);
    if selection.selected() != position {
        selection.set_selected(position);
    }
}

fn row_factory(
    browser: &RnWorkspaceBrowser,
    appwindow: &RnAppWindow,
) -> gtk4::SignalListItemFactory {
    let factory = gtk4::SignalListItemFactory::new();
    factory.connect_setup(clone!(
        #[weak]
        appwindow,
        #[weak]
        browser,
        move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let filerow = super::RnFileRow::new();
            filerow.init(&appwindow);
            add_tree_actions(&browser, &filerow);
            let expander = gtk4::TreeExpander::builder()
                .child(&filerow)
                .indent_for_icon(true)
                .build();
            expander.add_css_class("onepepe-tree-row");
            // drop a page or section on this row
            let drop = gtk4::DropTarget::new(gio::File::static_type(), gdk::DragAction::MOVE);
            drop.connect_drop(clone!(
                #[weak]
                browser,
                #[weak]
                filerow,
                #[upgrade_or]
                false,
                move |_, value, _, _| {
                    let (Ok(dragged), Some(target)) = (
                        value.get::<gio::File>(),
                        filerow.current_file().and_then(|f| f.path()),
                    ) else {
                        return false;
                    };
                    drop_on(&browser, &dragged, &target)
                }
            ));
            expander.add_controller(drop);
            list_item.set_child(Some(&expander));
        }
    ));
    factory.connect_bind(|_, list_item| {
        let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        let Some(expander) = list_item.child().and_downcast::<gtk4::TreeExpander>() else {
            return;
        };
        let Some(filerow) = expander.child().and_downcast::<super::RnFileRow>() else {
            return;
        };
        let row = list_item.item().and_downcast::<gtk4::TreeListRow>();
        expander.set_list_row(row.as_ref());
        let Some(info) = row.and_then(|r| r.item()).and_downcast::<gio::FileInfo>() else {
            return;
        };
        let Some(file) = info
            .attribute_object("standard::file")
            .and_downcast::<gio::File>()
        else {
            return;
        };
        let name = file
            .basename()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        filerow.file_label().set_label(&display_name(&name));
        if let Some(icon) = info.icon() {
            filerow.file_image().set_from_gicon(&icon);
        }
        filerow
            .drag_source()
            .set_content(Some(&gdk::ContentProvider::for_value(&file.to_value())));
        filerow.set_position(list_item.position());
        filerow.set_current_file(Some(file));
    });
    factory.connect_unbind(|_, list_item| {
        let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        if let Some(expander) = list_item.child().and_downcast::<gtk4::TreeExpander>() {
            expander.set_list_row(None);
        }
    });
    factory
}

/// The entries of the tree in the menu of a row: make subpage, promote, move up/down.
fn add_tree_actions(browser: &RnWorkspaceBrowser, filerow: &super::RnFileRow) {
    let group = filerow.imp().action_group.clone();
    let action = |name: &str, f: fn(&mut FolderIndex, &[Entry], &str) -> bool| {
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate(clone!(
            #[weak]
            browser,
            #[weak]
            filerow,
            move |_, _| {
                let Some(path) = filerow.current_file().and_then(|f| f.path()) else {
                    return;
                };
                let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
                    return;
                };
                let name = name.to_string_lossy().to_string();
                edit_index(&browser, dir, |index, entries| f(index, entries, &name));
            }
        ));
        group.add_action(&action);
    };
    action("tree-make-subpage", |index, entries, name| {
        index.make_subpage(entries, name)
    });
    action("tree-promote", |index, entries, name| {
        index.promote(entries, name)
    });
    action("tree-move-up", |index, entries, name| {
        move_by(index, entries, name, -1)
    });
    action("tree-move-down", |index, entries, name| {
        move_by(index, entries, name, 1)
    });

    if let Some(menu) = filerow
        .imp()
        .popovermenu
        .menu_model()
        .and_downcast::<gio::Menu>()
    {
        let section = gio::Menu::new();
        section.append(Some(&gettext("Move up")), Some("filerow.tree-move-up"));
        section.append(Some(&gettext("Move down")), Some("filerow.tree-move-down"));
        section.append(
            Some(&gettext("Make subpage")),
            Some("filerow.tree-make-subpage"),
        );
        section.append(
            Some(&gettext("Promote subpage")),
            Some("filerow.tree-promote"),
        );
        menu.append_section(None, &section);
    }
}

/// Moves a page one place up or down among the pages of its level.
fn move_by(index: &mut FolderIndex, entries: &[Entry], name: &str, step: i32) -> bool {
    let parent = index.parent_of(name, entries).map(str::to_string);
    let siblings = index.children(entries, parent.as_deref());
    let Some(i) = siblings.iter().position(|e| e.name == name) else {
        return false;
    };
    let target = i as i32 + step;
    if target < 0 || target as usize >= siblings.len() {
        return false;
    }
    // before the sibling above, or before the one after the sibling below
    let before = if step < 0 {
        Some(siblings[target as usize].name.clone())
    } else {
        siblings.get(target as usize + 1).map(|e| e.name.clone())
    };
    match before {
        Some(before) => index.move_before(entries, name, Some(&before)),
        None => {
            // last of its level: right after the last sibling
            let last = &siblings[siblings.len() - 1].name;
            let after_last = entries
                .iter()
                .skip_while(|e| e.name != *last)
                .nth(1)
                .map(|e| e.name.clone());
            index.move_before(entries, name, after_last.as_deref());
        }
    }
    true
}
