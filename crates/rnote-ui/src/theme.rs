// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the color tokens of the design guide, the only place with literal colors. From this table the app
// builds the CSS of the light, dark and high contrast themes: named colors `@op_*` that data/style.css uses, and
// the colors of libadwaita (both the named colors of libadwaita <= 1.5 and the CSS variables of libadwaita >= 1.6),
// so the standard widgets follow the same palette. The theme is reloaded when the system switches light/dark.

// Imports
use crate::RnAppWindow;
use adw::prelude::*;
use gtk4::{gdk, glib, glib::clone};
use std::cell::RefCell;

/// (token, light, dark). The table of the design guide, plus the text color on the header and on the brand color.
pub(crate) const TOKENS: &[(&str, &str, &str)] = &[
    ("bg_canvas", "#FFFFFF", "#292929"),
    ("bg_surface", "#FAFAFA", "#1F1F1F"),
    ("bg_frame", "#F0F0F0", "#141414"),
    ("bg_raised", "#FFFFFF", "#333333"),
    ("fg_1", "#242424", "#FFFFFF"),
    ("fg_2", "#424242", "#D6D6D6"),
    ("fg_3", "#616161", "#ADADAD"),
    ("fg_disabled", "#9E9E9E", "#6B6B6B"),
    ("stroke_1", "#D1D1D1", "#666666"),
    ("stroke_2", "#E0E0E0", "#4A4A4A"),
    ("header", "#5B1785", "#2A1638"),
    ("brand", "#7719AA", "#B889D6"),
    ("brand_hover", "#5F1388", "#CBA6E3"),
    ("brand_tint", "#F3E8F9", "#3A2447"),
    ("hover", "#F5F5F5", "#383838"),
    ("on_header", "#FFFFFF", "#FFFFFF"),
    ("on_brand", "#FFFFFF", "#1F1F1F"),
    ("canvas_cross", "#616161", "#ADADAD"),
];

/// High contrast: stronger lines and secondary text (light, dark).
const HIGH_CONTRAST: &[(&str, &str, &str)] = &[
    ("fg_3", "#424242", "#D6D6D6"),
    ("stroke_1", "#616161", "#ADADAD"),
    ("stroke_2", "#9E9E9E", "#808080"),
];

/// Minimal shadows, never pure black.
const SHADOW: (&str, &str) = ("rgba(20, 20, 20, 0.10)", "rgba(10, 10, 10, 0.45)");

/// libadwaita color -> token.
const ADW_COLORS: &[(&str, &str)] = &[
    ("accent_bg_color", "brand"),
    ("accent_color", "brand"),
    ("accent_fg_color", "on_brand"),
    ("window_bg_color", "bg_surface"),
    ("window_fg_color", "fg_1"),
    ("view_bg_color", "bg_canvas"),
    ("view_fg_color", "fg_1"),
    ("headerbar_bg_color", "bg_surface"),
    ("headerbar_fg_color", "fg_1"),
    ("headerbar_backdrop_color", "bg_surface"),
    ("headerbar_border_color", "stroke_2"),
    ("sidebar_bg_color", "bg_surface"),
    ("sidebar_fg_color", "fg_1"),
    ("sidebar_backdrop_color", "bg_surface"),
    ("secondary_sidebar_bg_color", "bg_surface"),
    ("secondary_sidebar_fg_color", "fg_1"),
    ("card_bg_color", "bg_raised"),
    ("card_fg_color", "fg_1"),
    ("popover_bg_color", "bg_raised"),
    ("popover_fg_color", "fg_1"),
    ("dialog_bg_color", "bg_surface"),
    ("dialog_fg_color", "fg_1"),
    ("thumbnail_bg_color", "bg_raised"),
    ("thumbnail_fg_color", "fg_1"),
];

/// The value of a token in a theme.
pub(crate) fn token(name: &str, dark: bool, high_contrast: bool) -> Option<&'static str> {
    let pick = |&(_, light, dark_value): &(&str, &'static str, &'static str)| {
        if dark { dark_value } else { light }
    };
    high_contrast
        .then(|| HIGH_CONTRAST.iter().find(|t| t.0 == name).map(pick))
        .flatten()
        .or_else(|| TOKENS.iter().find(|t| t.0 == name).map(pick))
}

/// The CSS with the colors of a theme.
pub(crate) fn theme_css(dark: bool, high_contrast: bool) -> String {
    let shadow = if dark { SHADOW.1 } else { SHADOW.0 };
    let mut css = String::new();
    for (name, ..) in TOKENS {
        css += &format!(
            "@define-color op_{name} {};\n",
            token(name, dark, high_contrast).unwrap()
        );
    }
    css += &format!("@define-color op_shadow {shadow};\n");
    for (adw, name) in ADW_COLORS {
        css += &format!("@define-color {adw} @op_{name};\n");
    }
    // libadwaita >= 1.6 uses CSS variables, which GTK supports since 4.16 (older versions would warn about them)
    if gtk4::check_version(4, 16, 0).is_some() {
        return css;
    }
    css += ":root {\n";
    for (adw, name) in ADW_COLORS {
        css += &format!(
            "    --{}: {};\n",
            adw.replace('_', "-"),
            token(name, dark, high_contrast).unwrap()
        );
    }
    css += "}\n";
    css
}

/// Relative luminance (WCAG 2).
#[cfg(test)]
pub(crate) fn luminance(hex: &str) -> f64 {
    let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    let channel = |c: u32| {
        let c = c as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel((v >> 16) & 0xff)
        + 0.7152 * channel((v >> 8) & 0xff)
        + 0.0722 * channel(v & 0xff)
}

/// Contrast ratio (WCAG 2) between two colors.
#[cfg(test)]
pub(crate) fn contrast(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// (text or control, background, minimum contrast) pairs that the themes must satisfy:
/// 4.5:1 for text, 3:1 for controls and the marks of the active state.
#[cfg(test)]
pub(crate) const CONTRAST_PAIRS: &[(&str, &str, f64)] = &[
    ("fg_1", "bg_canvas", 4.5),
    ("fg_1", "bg_surface", 4.5),
    ("fg_1", "bg_frame", 4.5),
    ("fg_1", "bg_raised", 4.5),
    ("fg_2", "bg_canvas", 4.5),
    ("fg_2", "bg_surface", 4.5),
    ("fg_2", "bg_raised", 4.5),
    ("fg_2", "hover", 4.5),
    ("fg_3", "bg_canvas", 4.5),
    ("fg_3", "bg_surface", 4.5),
    ("fg_3", "bg_raised", 4.5),
    ("fg_1", "brand_tint", 4.5),
    ("brand", "brand_tint", 4.5),
    ("brand", "bg_canvas", 4.5),
    ("brand", "bg_raised", 4.5),
    ("brand_hover", "bg_raised", 4.5),
    ("on_brand", "brand", 4.5),
    ("on_header", "header", 4.5),
    ("brand", "bg_surface", 3.0),
    ("canvas_cross", "bg_canvas", 3.0),
];

thread_local! {
    static PROVIDER: RefCell<Option<gtk4::CssProvider>> = const { RefCell::new(None) };
}

/// Loads the theme and follows the light/dark and high contrast switches of the system (and of the app setting).
pub(crate) fn setup(appwindow: &RnAppWindow) {
    let style_manager = adw::StyleManager::default();
    let apply = clone!(
        #[weak]
        appwindow,
        move |style_manager: &adw::StyleManager| apply(&appwindow, style_manager)
    );
    style_manager.connect_dark_notify(apply.clone());
    style_manager.connect_high_contrast_notify(apply.clone());
    apply(&style_manager);
}

fn apply(appwindow: &RnAppWindow, style_manager: &adw::StyleManager) {
    let (dark, high_contrast) = (style_manager.is_dark(), style_manager.is_high_contrast());
    PROVIDER.with_borrow_mut(|provider| {
        let provider = provider.get_or_insert_with(|| {
            let provider = gtk4::CssProvider::new();
            if let Some(display) = gdk::Display::default() {
                // below the app stylesheet, so style.css can use the `@op_*` colors; above the libadwaita theme
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION - 1,
                );
            }
            provider
        });
        provider.load_from_string(&theme_css(dark, high_contrast));
    });
    for (class, on) in [("dark", dark), ("high-contrast", high_contrast)] {
        if on {
            appwindow.add_css_class(class);
        } else {
            appwindow.remove_css_class(class);
        }
    }
    // the canvas follows the theme (see canvas_dark())
    appwindow.queue_draw();
    glib::idle_add_local_once(clone!(
        #[weak]
        appwindow,
        move || {
            for page in appwindow.tabs_snapshot() {
                if let Some(wrapper) = page.child().downcast_ref::<crate::RnCanvasWrapper>() {
                    wrapper.canvas().queue_draw();
                }
            }
        }
    ));
}

/// Whether the canvas is shown in dark colors: when the theme is dark, unless turned off in View.
/// Only on screen: the files and the exports keep their colors.
pub(crate) fn canvas_dark() -> bool {
    adw::StyleManager::default().is_dark() && crate::prefs::get_bool("lienzo-oscuro", true)
}

/// The color matrix that shows the canvas in dark: it flips the lightness and keeps the hue, so white paper is
/// shown as bg-canvas dark (#292929) and black ink as #E8E8E8; a red stroke stays red, lighter.
pub(crate) fn canvas_dark_matrix() -> (gtk4::graphene::Matrix, gtk4::graphene::Vec4) {
    // out = in + (L' - L), with L' = hi - (hi - lo)·L, the luminance L = 0.2126 r + 0.7152 g + 0.0722 b
    let (lo, hi) = (41.0 / 255.0, 232.0 / 255.0);
    let k = 1.0 + (hi - lo);
    let w = [0.2126_f32, 0.7152, 0.0722];
    let mut m = [0.0_f32; 16];
    // graphene: the color is multiplied as a row vector (out = in · M), M[row = input][col = output]
    for input in 0..3 {
        for output in 0..3 {
            m[input * 4 + output] = if input == output { 1.0 } else { 0.0 } - k * w[input];
        }
    }
    m[15] = 1.0;
    (
        gtk4::graphene::Matrix::from_float(m),
        gtk4::graphene::Vec4::new(hi, hi, hi, 0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_of_the_themes() {
        let mut failures = vec![];
        for dark in [false, true] {
            for high_contrast in [false, true] {
                for &(fg, bg, min) in CONTRAST_PAIRS {
                    let ratio = contrast(
                        token(fg, dark, high_contrast).unwrap(),
                        token(bg, dark, high_contrast).unwrap(),
                    );
                    if ratio < min {
                        failures.push(format!(
                            "{fg} on {bg} (dark {dark}, hc {high_contrast}): {ratio:.2} < {min}"
                        ));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn no_loose_colors_in_the_stylesheet() {
        // all the colors of the app stylesheet come from the tokens (theme.rs), except the checkerboard pattern
        // of the color buttons, which shows transparency
        let css = include_str!("../data/style.css");
        let loose: Vec<&str> = css
            .lines()
            .filter(|l| !l.contains("op-allow-literal"))
            .filter(|l| {
                l.contains('#')
                    && l.split('#')
                        .skip(1)
                        .any(|s| s.starts_with(|c: char| c.is_ascii_hexdigit()))
                    || l.contains("rgb(")
                    || l.contains("rgba(")
            })
            .collect();
        assert!(loose.is_empty(), "loose colors in style.css: {loose:#?}");
    }

    #[test]
    fn dark_matrix_maps_paper_and_ink() {
        let (lo, hi) = (41.0 / 255.0, 232.0 / 255.0);
        let k = 1.0 + (hi - lo);
        let map = |c: [f64; 3]| {
            let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            c.map(|v| (v - k * l + hi).clamp(0.0, 1.0))
        };
        let white = map([1.0, 1.0, 1.0]);
        let black = map([0.0, 0.0, 0.0]);
        assert!((white[0] - lo).abs() < 1e-6 && (black[0] - hi).abs() < 1e-6);
        let red = map([1.0, 0.0, 0.0]);
        assert!(red[0] > red[1] && red[0] > red[2]);
    }
}
