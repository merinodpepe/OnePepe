// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the frame of the window from the design guide: the logo in the header, the 24 px status bar below the
// canvas (page size, pages, zoom, saved) and the faint logo on an empty document.

// Imports
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};
use rnote_engine::engine::onepepe_insert::DocStatus;
use std::time::Duration;
use tracing::error;

pub(crate) fn setup(appwindow: &RnAppWindow) {
    // the monochrome logo at the left of the header
    let logo = gtk4::Image::builder()
        .icon_name("onepepe-logo-symbolic")
        .pixel_size(24)
        .tooltip_text("OnePepe")
        .margin_start(4)
        .margin_end(4)
        .build();
    logo.add_css_class("onepepe-logo");
    match appwindow
        .main_header()
        .left_sidebar_reveal_toggle()
        .parent()
        .and_then(|w| w.parent())
        .and_downcast::<gtk4::Box>()
    {
        Some(quickactions) => quickactions.prepend(&logo),
        None => error!("Could not place the logo in the header"),
    }

    // the faint logo on an empty document
    let banner = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .can_target(false)
        .build();
    banner.append(
        &gtk4::Image::builder()
            .icon_name("onepepe-logo-symbolic")
            .pixel_size(160)
            .build(),
    );
    let banner_label = gtk4::Label::new(Some("OnePepe"));
    banner_label.add_css_class("title-1");
    banner.append(&banner_label);
    banner.add_css_class("onepepe-banner");
    appwindow
        .overlays()
        .imp()
        .toolbar_overlay
        .add_overlay(&banner);

    // the status bar
    let page = gtk4::Label::new(None);
    let zoom = gtk4::Label::new(None);
    let saved = gtk4::Label::new(None);
    let bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    bar.add_css_class("onepepe-statusbar");
    bar.append(&page);
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bar.append(&spacer);
    bar.append(&saved);
    bar.append(&zoom);
    // OnePepe: the version, to know which one is installed when something fails
    let version = gtk4::Label::new(Some(&format!(
        "OnePepe {}",
        crate::globals::onepepe_version()
    )));
    version.add_css_class("dim-label");
    bar.append(&version);
    match appwindow
        .imp()
        .tabbar
        .get()
        .parent()
        .and_downcast::<gtk4::Box>()
    {
        Some(content_box) => content_box.append(&bar),
        None => error!("Could not place the status bar"),
    }

    // there is no single signal for all of it (zoom, pages, saving, switching tabs): a cheap refresh 4 times a second
    glib::timeout_add_local(
        Duration::from_millis(250),
        clone!(
            #[weak]
            appwindow,
            #[weak]
            bar,
            #[weak]
            banner,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    bar.set_visible(false);
                    banner.set_visible(false);
                    return glib::ControlFlow::Continue;
                };
                let status = canvas.engine_ref().doc_status();
                bar.set_visible(!appwindow.focus_mode());
                banner.set_visible(status.strokes == 0);
                set_text(&page, &page_text(&status));
                set_text(&zoom, &format!("{:.0} %", status.zoom * 100.0));
                set_text(
                    &saved,
                    &if canvas.unsaved_changes() {
                        gettext("Unsaved changes")
                    } else if canvas.output_file().is_some() {
                        gettext("Saved")
                    } else {
                        gettext("Draft")
                    },
                );
                glib::ControlFlow::Continue
            }
        ),
    );
}

fn set_text(label: &gtk4::Label, text: &str) {
    if label.label() != text {
        label.set_label(text);
    }
}

fn page_text(status: &DocStatus) -> String {
    let size = match status.page_size {
        Some((size, false)) => size.name().to_string(),
        Some((size, true)) => gettext("{} landscape").replace("{}", size.name()),
        None => gettext("Custom size"),
    };
    if status.pages > 1 {
        gettext("{size} · {n} pages")
            .replace("{size}", &size)
            .replace("{n}", &status.pages.to_string())
    } else {
        size
    }
}
