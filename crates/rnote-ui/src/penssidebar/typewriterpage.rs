// Imports
use crate::RnAppWindow;
use gtk4::{
    Button, CompositeTemplate, EmojiChooser, FontDialog, MenuButton, SpinButton, ToggleButton,
    Widget, glib, glib::clone, pango, prelude::*, subclass::prelude::*,
};
use rnote_engine::strokes::textstroke::{TextAlignment, TextAttribute, TextStyle};
use std::cell::RefCell;
use tracing::debug;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/typewriterpage.ui")]
    pub(crate) struct RnTypewriterPage {
        pub(super) prev_picked_font_family: RefCell<Option<pango::FontFamily>>,

        #[template_child]
        pub(crate) fontdialog_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) font_size_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) emojichooser_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) emojichooser: TemplateChild<EmojiChooser>,
        #[template_child]
        pub(crate) text_reset_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_strikethrough_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_superscript_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) symbolcatalog_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) datetime_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) text_subscript_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_list_bullet_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_list_numbered_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_list_checkbox_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_align_start_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_center_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_end_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_fill_togglebutton: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnTypewriterPage {
        const NAME: &'static str = "RnTypewriterPage";
        type Type = super::RnTypewriterPage;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnTypewriterPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnTypewriterPage {}
}

glib::wrapper! {
    pub(crate) struct RnTypewriterPage(ObjectSubclass<imp::RnTypewriterPage>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnTypewriterPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnTypewriterPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn emojichooser_menubutton(&self) -> MenuButton {
        self.imp().emojichooser_menubutton.get()
    }

    #[allow(unused)]
    pub(crate) fn alignment(&self) -> Option<TextAlignment> {
        if self.imp().text_align_start_togglebutton.is_active() {
            Some(TextAlignment::Start)
        } else if self.imp().text_align_center_togglebutton.is_active() {
            Some(TextAlignment::Center)
        } else if self.imp().text_align_end_togglebutton.is_active() {
            Some(TextAlignment::End)
        } else if self.imp().text_align_fill_togglebutton.is_active() {
            Some(TextAlignment::Fill)
        } else {
            None
        }
    }

    pub(crate) fn set_alignment(&self, alignment: TextAlignment) {
        match alignment {
            TextAlignment::Start => self.imp().text_align_start_togglebutton.set_active(true),
            TextAlignment::Center => self.imp().text_align_center_togglebutton.set_active(true),
            TextAlignment::End => self.imp().text_align_end_togglebutton.set_active(true),
            TextAlignment::Fill => self.imp().text_align_fill_togglebutton.set_active(true),
        }
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.fontdialog_button.connect_clicked(clone!(#[weak(rename_to=typewriterpage)] self , #[weak] appwindow , move |_| {
            glib::spawn_future_local(clone!(#[weak] typewriterpage, #[weak] appwindow , async move {
                let dialog = FontDialog::builder().modal(false).build();
                let prev_picked_font_family = typewriterpage.imp().prev_picked_font_family.borrow().clone();
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                match dialog.choose_family_future(Some(&appwindow), prev_picked_font_family.as_ref()).await {
                    Ok(new_font_family) => {
                        let font_family_name = new_font_family.name().to_string();

                        typewriterpage.imp().prev_picked_font_family.borrow_mut().replace(new_font_family);
                        appwindow.engine_config().write().pens_config.typewriter_config.text_style.font_family.clone_from(&font_family_name);
                        let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {style.font_family = font_family_name});
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                    Err(e) => debug!("Did not choose new font family (Error or dialog dismissed by user), Err: {e:?}"),
                }
            }));
        }));

        // Font size
        imp.font_size_spinbutton.set_increments(1.0, 5.0);
        imp.font_size_spinbutton
            .set_range(TextStyle::FONT_SIZE_MIN, TextStyle::FONT_SIZE_MAX);
        imp.font_size_spinbutton
            .set_value(TextStyle::FONT_SIZE_DEFAULT);

        imp.font_size_spinbutton.connect_value_changed(clone!(
            #[weak]
            appwindow,
            move |spinbutton| {
                let font_size = spinbutton.value();
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .typewriter_config
                    .text_style
                    .font_size = font_size;
                let widget_flags = canvas
                    .engine_mut()
                    .text_selection_change_style(|style| style.font_size = font_size);
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Emojis
        imp.emojichooser.connect_emoji_picked(clone!(
            #[weak]
            appwindow,
            move |_, emoji_str| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().insert_text(emoji_str.to_string(), None);
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // reset
        imp.text_reset_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().text_selection_remove_attributes();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Strikethrough
        imp.text_strikethrough_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas
                    .engine_mut()
                    .text_selection_toggle_attribute(TextAttribute::Strikethrough(true));
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // OnePepe: symbol catalog
        imp.symbolcatalog_menubutton
            .set_popover(Some(&symbol_catalog_popover(appwindow)));

        // OnePepe: date / time stamp
        imp.datetime_menubutton
            .set_popover(Some(&date_time_popover(appwindow)));

        // OnePepe: lists and checkboxes
        {
            use rnote_engine::pens::typewriter::scripts::ListKind;
            for (button, kind) in [
                (imp.text_list_bullet_button.get(), ListKind::Bullet),
                (imp.text_list_numbered_button.get(), ListKind::Numbered),
                (imp.text_list_checkbox_button.get(), ListKind::Checkbox),
            ] {
                button.connect_clicked(clone!(
                    #[weak]
                    appwindow,
                    move |_| {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        let widget_flags = canvas.engine_mut().text_paragraph_list(kind);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                ));
            }
        }

        // OnePepe: superscript / subscript
        for (button, sup) in [
            (imp.text_superscript_button.get(), true),
            (imp.text_subscript_button.get(), false),
        ] {
            button.connect_clicked(clone!(
                #[weak]
                appwindow,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let widget_flags = canvas.engine_mut().text_selection_toggle_script(sup);
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));
        }

        // Alignment
        imp.text_align_start_togglebutton
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |togglebutton| {
                    if !togglebutton.is_active() {
                        return;
                    }
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .typewriter_config
                        .text_style
                        .alignment = TextAlignment::Start;
                    let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {
                        style.alignment = TextAlignment::Start
                    });
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));

        imp.text_align_center_togglebutton
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |togglebutton| {
                    if !togglebutton.is_active() {
                        return;
                    }
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .typewriter_config
                        .text_style
                        .alignment = TextAlignment::Center;
                    let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {
                        style.alignment = TextAlignment::Center
                    });
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));

        imp.text_align_end_togglebutton
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |togglebutton| {
                    if !togglebutton.is_active() {
                        return;
                    }
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .typewriter_config
                        .text_style
                        .alignment = TextAlignment::End;
                    let widget_flags = canvas
                        .engine_mut()
                        .text_selection_change_style(|style| style.alignment = TextAlignment::End);
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));

        imp.text_align_fill_togglebutton
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |togglebutton| {
                    if !togglebutton.is_active() {
                        return;
                    }
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .typewriter_config
                        .text_style
                        .alignment = TextAlignment::Fill;
                    let widget_flags = canvas
                        .engine_mut()
                        .text_selection_change_style(|style| style.alignment = TextAlignment::Fill);
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        let typewriter_config = appwindow
            .engine_config()
            .read()
            .pens_config
            .typewriter_config
            .clone();

        imp.font_size_spinbutton
            .set_value(typewriter_config.text_style.font_size);

        self.set_alignment(typewriter_config.text_style.alignment);
    }
}

/// OnePepe: symbols that can be inserted with one click, by category.
const SYMBOL_CATALOG: &[(&str, &str)] = &[
    ("Greek", "αβγδεζηθικλμνξοπρστυφχψω"),
    ("Greek (Capitals)", "ΓΔΘΛΞΠΣΦΨΩ"),
    ("Operators", "±∓×÷·∘≈≠≡≤≥≪≫∝∼→←↔⇒⇔↑↓"),
    ("Analysis", "∂∇∫∬∮∑∏√∛∞′″°"),
    ("Sets and Logic", "∈∉⊂⊆∪∩∅∀∃¬∧∨ℕℤℚℝℂ"),
    ("Physics", "ℏÅµΩ‰⊥∥∠△"),
];

/// OnePepe: the popover of the symbol catalog. A click inserts the symbol into the text.
fn symbol_catalog_popover(appwindow: &RnAppWindow) -> gtk4::Popover {
    let popover = gtk4::Popover::new();
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    for (title, symbols) in SYMBOL_CATALOG {
        let label = gtk4::Label::new(Some(&gettextrs::gettext(*title)));
        label.set_halign(gtk4::Align::Start);
        label.add_css_class("caption-heading");
        vbox.append(&label);
        let flow = gtk4::FlowBox::builder()
            .max_children_per_line(10)
            .min_children_per_line(10)
            .selection_mode(gtk4::SelectionMode::None)
            .build();
        for symbol in symbols.chars() {
            let button = gtk4::Button::with_label(&symbol.to_string());
            button.add_css_class("flat");
            button.connect_clicked(clone!(
                #[weak]
                appwindow,
                #[weak]
                popover,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let widget_flags = canvas.engine_mut().insert_text(symbol.to_string(), None);
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                    popover.popdown();
                }
            ));
            flow.append(&button);
        }
        vbox.append(&flow);
    }
    popover.set_child(Some(&vbox));
    popover
}

/// OnePepe: the popover to insert the date, the time or both.
fn date_time_popover(appwindow: &RnAppWindow) -> gtk4::Popover {
    use rnote_engine::pens::typewriter::scripts::DateTimeStamp;

    let popover = gtk4::Popover::new();
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
    for (label, kind) in [
        (gettextrs::gettext("Date"), DateTimeStamp::Date),
        (gettextrs::gettext("Time"), DateTimeStamp::Time),
        (
            gettextrs::gettext("Date and Time"),
            DateTimeStamp::DateAndTime,
        ),
    ] {
        let button = gtk4::Button::with_label(&label);
        button.add_css_class("flat");
        button.connect_clicked(clone!(
            #[weak]
            appwindow,
            #[weak]
            popover,
            move |_| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().insert_date_time(kind);
                appwindow.handle_widget_flags(widget_flags, &canvas);
                popover.popdown();
            }
        ));
        vbox.append(&button);
    }
    popover.set_child(Some(&vbox));
    popover
}
