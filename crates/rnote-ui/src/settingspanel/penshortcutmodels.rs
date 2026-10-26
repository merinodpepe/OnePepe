// Imports
use gettextrs::gettext;
use gtk4::{Align, StringObject};
use gtk4::{
    Image, Label, ListItem, Orientation, SignalListItemFactory, StringList, glib::prelude::*,
    prelude::*,
};
use rnote_engine::pens::PenStyle;
use rnote_engine::pens::shortcuts::ShortcutPenVariant;
use std::ops::{Deref, DerefMut};

/// OnePepe: the entries of the shortcut dropdown: an id, the pen style and the pen variant.
pub(crate) const SHORTCUT_ENTRIES: [(&str, PenStyle, ShortcutPenVariant); 9] = [
    ("brush", PenStyle::Brush, ShortcutPenVariant::Current),
    ("shaper", PenStyle::Shaper, ShortcutPenVariant::Current),
    (
        "typewriter",
        PenStyle::Typewriter,
        ShortcutPenVariant::Current,
    ),
    ("eraser", PenStyle::Eraser, ShortcutPenVariant::Current),
    ("selector", PenStyle::Selector, ShortcutPenVariant::Current),
    ("tools", PenStyle::Tools, ShortcutPenVariant::Current),
    (
        "eraser-whole-strokes",
        PenStyle::Eraser,
        ShortcutPenVariant::EraserWholeStrokes,
    ),
    (
        "eraser-area",
        PenStyle::Eraser,
        ShortcutPenVariant::EraserArea,
    ),
    (
        "selector-lasso",
        PenStyle::Selector,
        ShortcutPenVariant::SelectorLasso,
    ),
];

/// OnePepe: the index of an entry, or the entry of the pen style when the variant is not listed.
pub(crate) fn shortcut_entry_index(style: PenStyle, variant: ShortcutPenVariant) -> u32 {
    SHORTCUT_ENTRIES
        .iter()
        .position(|(_, s, v)| *s == style && *v == variant)
        .or_else(|| {
            SHORTCUT_ENTRIES
                .iter()
                .position(|(_, s, v)| *s == style && *v == ShortcutPenVariant::Current)
        })
        .unwrap_or(0) as u32
}

/// OnePepe: pen style, variant and label of an entry id.
fn shortcut_entry(id: &str) -> (PenStyle, ShortcutPenVariant, String) {
    let (_, style, variant) = SHORTCUT_ENTRIES
        .iter()
        .find(|(i, _, _)| *i == id)
        .copied()
        .unwrap_or(SHORTCUT_ENTRIES[0]);
    let label = match (style, variant) {
        (_, ShortcutPenVariant::EraserWholeStrokes) => gettext("Eraser (Whole Strokes)"),
        (_, ShortcutPenVariant::EraserArea) => gettext("Eraser (Area)"),
        (_, ShortcutPenVariant::SelectorLasso) => gettext("Selector (Lasso)"),
        (PenStyle::Brush, _) => gettext("Brush"),
        (PenStyle::Shaper, _) => gettext("Shaper"),
        (PenStyle::Typewriter, _) => gettext("Typewriter"),
        (PenStyle::Eraser, _) => gettext("Eraser"),
        (PenStyle::Selector, _) => gettext("Selector"),
        (PenStyle::Tools, _) => gettext("Tools"),
    };
    (style, variant, label)
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleListModel(StringList);

impl Default for ChangePenStyleListModel {
    /// Only the pen styles (the first entries, in the order of `PenStyle`).
    fn default() -> Self {
        Self(StringList::new(
            &SHORTCUT_ENTRIES
                .iter()
                .filter(|(_, _, v)| *v == ShortcutPenVariant::Current)
                .map(|(id, _, _)| *id)
                .collect::<Vec<&str>>(),
        ))
    }
}

impl ChangePenStyleListModel {
    /// OnePepe: the pen styles and the pen variants, for the shortcut rows.
    pub(crate) fn with_variants() -> Self {
        Self(StringList::new(
            &SHORTCUT_ENTRIES
                .iter()
                .map(|(id, _, _)| *id)
                .collect::<Vec<&str>>(),
        ))
    }
}

impl Deref for ChangePenStyleListModel {
    type Target = StringList;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleListModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleListFactory(SignalListItemFactory);

impl Default for ChangePenStyleListFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let item_box = gtk4::Box::builder()
                .orientation(Orientation::Horizontal)
                .build();
            let image = Image::builder().margin_end(12).halign(Align::Start).build();
            let label = Label::builder()
                .label("")
                .hexpand(true)
                .halign(Align::Start)
                .build();

            item_box.prepend(&image);
            item_box.append(&label);
            list_item.set_child(Some(&item_box));
        });
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let (pen_style, _, entry_label) = shortcut_entry(
                &list_item
                    .item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string(),
            );
            let item_box = list_item.child().unwrap().downcast::<gtk4::Box>().unwrap();

            let mut child = item_box.first_child();
            while let Some(ref next_child) = child {
                if next_child.type_() == Label::static_type() {
                    next_child
                        .downcast_ref::<Label>()
                        .unwrap()
                        .set_label(&entry_label);
                } else if next_child.type_() == Image::static_type() {
                    next_child
                        .downcast_ref::<Image>()
                        .unwrap()
                        .set_icon_name(Some(pen_style.icon_name().as_str()));
                }

                child = next_child.next_sibling();
            }
        });
        Self(factory)
    }
}

impl Deref for ChangePenStyleListFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleListFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleIconFactory(SignalListItemFactory);

impl Default for ChangePenStyleIconFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let image = Image::builder().build();
            list_item.set_child(Some(&image));
        });
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let (pen_style, _, _) = shortcut_entry(
                &list_item
                    .item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string(),
            );
            let image = list_item.child().unwrap().downcast::<Image>().unwrap();
            image
                .downcast_ref::<Image>()
                .unwrap()
                .set_icon_name(Some(pen_style.icon_name().as_str()));
        });
        Self(factory)
    }
}

impl Deref for ChangePenStyleIconFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleIconFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
