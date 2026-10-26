// Imports
use super::penshortcutmodels::{
    ChangePenStyleIconFactory, ChangePenStyleListFactory, ChangePenStyleListModel,
    SHORTCUT_ENTRIES, shortcut_entry_index,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{CompositeTemplate, DropDown, ListBoxRow, Widget, glib, glib::clone, glib::subclass::*};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use rnote_engine::pens::PenStyle;
use rnote_engine::pens::shortcuts::ShortcutAction;
use rnote_engine::pens::shortcuts::{ShortcutMode, ShortcutPenVariant};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penshortcutrow.ui")]
    pub(crate) struct RnPenShortcutRow {
        pub(crate) action: RefCell<ShortcutAction>,
        pub(crate) changepenstyle_model: ChangePenStyleListModel,

        #[template_child]
        pub(crate) mode_dropdown: TemplateChild<DropDown>,
    }

    impl Default for RnPenShortcutRow {
        fn default() -> Self {
            Self {
                action: RefCell::new(ShortcutAction::ChangePenStyle {
                    style: PenStyle::Eraser,
                    mode: ShortcutMode::Temporary,
                    variant: ShortcutPenVariant::Current,
                }),
                changepenstyle_model: ChangePenStyleListModel::with_variants(),

                mode_dropdown: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnPenShortcutRow {
        const NAME: &'static str = "RnPenShortcutRow";
        type Type = super::RnPenShortcutRow;
        type ParentType = adw::ComboRow;
        type Interfaces = ();

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnPenShortcutRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let list_factory = ChangePenStyleListFactory::default();
            let icon_factory = ChangePenStyleIconFactory::default();

            obj.set_model(Some(&*self.changepenstyle_model));
            obj.set_list_factory(Some(&*list_factory));
            obj.set_factory(Some(&*icon_factory));

            obj.connect_selected_item_notify(move |row| {
                let (new_pen_style, new_variant) = row.pen_style_variant();

                match &mut *row.imp().action.borrow_mut() {
                    ShortcutAction::ChangePenStyle { style, variant, .. } => {
                        *style = new_pen_style;
                        *variant = new_variant;
                    }
                }
                row.emit_by_name::<()>("action-changed", &[]);
            });

            self.mode_dropdown.get().connect_selected_notify(clone!(
                #[weak(rename_to=penshortcutrow)]
                obj,
                move |_| {
                    match &mut *penshortcutrow.imp().action.borrow_mut() {
                        ShortcutAction::ChangePenStyle { mode, .. } => {
                            *mode = penshortcutrow.shortcut_mode();
                        }
                    }
                    penshortcutrow.emit_by_name::<()>("action-changed", &[]);
                }
            ));

            obj.connect_local(
                "action-changed",
                false,
                clone!(
                    #[weak(rename_to=penshortcutrow)]
                    obj,
                    #[upgrade_or]
                    None,
                    move |_values| {
                        penshortcutrow.update_ui();
                        None
                    }
                ),
            );
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("action-changed").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for RnPenShortcutRow {}
    impl ListBoxRowImpl for RnPenShortcutRow {}
    impl PreferencesRowImpl for RnPenShortcutRow {}
    impl ActionRowImpl for RnPenShortcutRow {}
    impl ComboRowImpl for RnPenShortcutRow {}
}

glib::wrapper! {
    pub(crate) struct RnPenShortcutRow(ObjectSubclass<imp::RnPenShortcutRow>)
        @extends adw::ComboRow, adw::ActionRow, adw::PreferencesRow, ListBoxRow, Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl RnPenShortcutRow {
    #[allow(clippy::new_without_default)]
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn action(&self) -> ShortcutAction {
        *self.imp().action.borrow()
    }

    #[allow(unused)]
    pub(crate) fn set_action(&self, action: ShortcutAction) {
        *self.imp().action.borrow_mut() = action;
        self.emit_by_name::<()>("action-changed", &[]);
    }

    /// OnePepe: the selected pen style and its variant.
    pub(crate) fn pen_style_variant(&self) -> (PenStyle, ShortcutPenVariant) {
        let (_, style, variant) = SHORTCUT_ENTRIES
            .get(self.selected() as usize)
            .copied()
            .unwrap_or(SHORTCUT_ENTRIES[0]);
        (style, variant)
    }

    pub(crate) fn set_pen_style_variant(&self, style: PenStyle, variant: ShortcutPenVariant) {
        self.set_selected(shortcut_entry_index(style, variant))
    }

    pub(crate) fn shortcut_mode(&self) -> ShortcutMode {
        ShortcutMode::try_from(self.imp().mode_dropdown.selected()).unwrap()
    }

    pub(crate) fn set_shortcut_mode(&self, mode: ShortcutMode) {
        self.imp()
            .mode_dropdown
            .set_selected(mode.to_u32().unwrap())
    }

    fn update_ui(&self) {
        match self.action() {
            ShortcutAction::ChangePenStyle {
                style,
                mode,
                variant,
            } => {
                self.set_pen_style_variant(style, variant);
                self.set_shortcut_mode(mode);
            }
        }
    }
}
