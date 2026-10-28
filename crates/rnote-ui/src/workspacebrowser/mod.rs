// Modules
mod filerow;
pub(crate) mod tree;
mod widgethelper;
pub(crate) mod workspaceactions;
pub(crate) mod workspacesbar;

// Re-exports
pub(crate) use filerow::RnFileRow;
pub(crate) use workspacesbar::RnWorkspacesBar;

// Imports
use crate::appwindow::RnAppWindow;
use gtk4::{
    Button, CompositeTemplate, DirectoryList, Grid, Label, ListView, ScrolledWindow, Separator,
    SingleSelection, Widget, gio, glib, glib::clone, prelude::*, subclass::prelude::*,
};
use std::path::PathBuf;
use tracing::warn;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacebrowser.ui")]
    pub(crate) struct RnWorkspaceBrowser {
        pub(crate) action_group: gio::SimpleActionGroup,
        pub(crate) dir_list: DirectoryList,

        #[template_child]
        pub(crate) grid: TemplateChild<Grid>,
        #[template_child]
        pub(crate) dir_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) corner_filler: TemplateChild<Separator>,
        #[template_child]
        pub(crate) files_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) files_listview: TemplateChild<ListView>,
        #[template_child]
        pub(crate) active_workspace_name_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) active_workspace_dir_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) dir_controls_dir_up_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) dir_controls_actions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) workspacesbar: TemplateChild<RnWorkspacesBar>,
    }

    impl Default for RnWorkspaceBrowser {
        fn default() -> Self {
            let dir_list = DirectoryList::new(Some("standard::*"), None as Option<&gio::File>);
            dir_list.set_monitored(true);

            Self {
                action_group: gio::SimpleActionGroup::new(),
                dir_list,

                grid: TemplateChild::<Grid>::default(),
                dir_box: TemplateChild::<gtk4::Box>::default(),
                corner_filler: TemplateChild::<Separator>::default(),
                files_scroller: TemplateChild::<ScrolledWindow>::default(),
                files_listview: TemplateChild::<ListView>::default(),
                active_workspace_name_label: TemplateChild::<Label>::default(),
                active_workspace_dir_label: TemplateChild::<Label>::default(),
                dir_controls_dir_up_button: TemplateChild::<Button>::default(),
                dir_controls_actions_box: TemplateChild::<gtk4::Box>::default(),
                workspacesbar: TemplateChild::<RnWorkspacesBar>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnWorkspaceBrowser {
        const NAME: &'static str = "RnWorkspaceBrowser";
        type Type = super::RnWorkspaceBrowser;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnWorkspaceBrowser {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj()
                .insert_action_group("workspacebrowser", Some(&self.action_group));
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnWorkspaceBrowser {}
}

glib::wrapper! {
    pub(crate) struct RnWorkspaceBrowser(ObjectSubclass<imp::RnWorkspaceBrowser>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnWorkspaceBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl RnWorkspaceBrowser {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn grid(&self) -> Grid {
        self.imp().grid.clone()
    }

    pub(crate) fn dir_box(&self) -> gtk4::Box {
        self.imp().dir_box.clone()
    }

    pub(crate) fn corner_filler(&self) -> Separator {
        self.imp().corner_filler.clone()
    }

    pub(crate) fn files_scroller(&self) -> ScrolledWindow {
        self.imp().files_scroller.clone()
    }

    pub(crate) fn workspacesbar(&self) -> RnWorkspacesBar {
        self.imp().workspacesbar.clone()
    }

    pub(crate) fn active_workspace_name_label(&self) -> Label {
        self.imp().active_workspace_name_label.clone()
    }

    pub(crate) fn active_workspace_dir_label(&self) -> Label {
        self.imp().active_workspace_dir_label.clone()
    }

    pub(crate) fn dir_controls_actions_box(&self) -> gtk4::Box {
        self.imp().dir_controls_actions_box.clone()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.imp().workspacesbar.get().init(appwindow);

        self.setup_dir_controls(appwindow);
        self.setup_files_list(appwindow);
        self.setup_actions(appwindow);
    }

    pub(crate) fn dir_list_file(&self) -> Option<gio::File> {
        self.imp().dir_list.file()
    }

    pub(crate) fn set_dir_list_file(&self, file: Option<&gio::File>) {
        self.imp().dir_list.set_file(file);
    }

    pub(crate) fn dir_list_dir(&self) -> Option<PathBuf> {
        self.imp().dir_list.file().and_then(|f| f.path())
    }

    pub(crate) fn refresh_dir_list_selected_workspace(&self) {
        if let Some(current_workspace_dir) = self
            .workspacesbar()
            .selected_workspacelistentry()
            .map(|e| PathBuf::from(e.dir()))
        {
            self.imp()
                .dir_list
                .set_file(Some(&gio::File::for_path(current_workspace_dir)));
        }
    }

    fn setup_actions(&self, appwindow: &RnAppWindow) {
        self.imp()
            .action_group
            .add_action(&workspaceactions::create_folder(self, appwindow));
        self.imp()
            .action_group
            .add_action(&workspaceactions::create_page(self, appwindow));
        self.imp()
            .action_group
            .add_action(&workspaceactions::open_folder(self, appwindow));
    }

    fn setup_dir_controls(&self, _appwindow: &RnAppWindow) {
        self.imp().dir_controls_dir_up_button.connect_clicked(clone!(#[weak(rename_to=workspacebrowser)] self , move |_| {
            if let Some(mut dir) = workspacebrowser.workspacesbar().selected_workspacelistentry().map(|e| PathBuf::from(e.dir())) {
                // don't canonicalize on windows, because that would convert the path to one with extended length syntax
                if !cfg!(target_os = "windows") {
                    dir = match dir.canonicalize() {
                        Ok(dir) => dir,
                        Err(e) => {
                            warn!("Could not canonicalize dir {dir:?} from workspacelistentry, Err: {e:?}");
                            return;
                        }
                    };
                }
                if let Some(parent) = dir.parent().map(|p| p.to_path_buf()) {
                    workspacebrowser.workspacesbar().set_selected_workspace_dir(parent);
                } else {
                    warn!("Can't move directory up from dir {dir:?} from workspacelistentry, has no parent.");
                }
            }
        }));
    }

    fn setup_files_list(&self, appwindow: &RnAppWindow) {
        // OnePepe: the tree of the notebook (tree.rs) instead of the flat list of the folder
        tree::setup(self, appwindow);

        self.imp().dir_list.connect_file_notify(clone!(
            #[weak(rename_to=workspacebrowser)]
            self,
            move |dir_list| {
                // Disable the dir up row when no file is set or has no parent.
                workspacebrowser
                    .imp()
                    .dir_controls_dir_up_button
                    .set_sensitive(dir_list.file().and_then(|f| f.parent()).is_some());
            }
        ));
    }

    /// Set the selected file in the files list with its position.
    pub(crate) fn files_list_set_selected(&self, position: Option<u32>) {
        if let Some(selection) = self
            .imp()
            .files_listview
            .model()
            .and_downcast::<SingleSelection>()
        {
            selection.set_selected(position.unwrap_or(gtk4::INVALID_LIST_POSITION));
        }
    }
}
