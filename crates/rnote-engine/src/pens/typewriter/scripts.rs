//! OnePepe: superscript and subscript text using the Unicode superscript and subscript characters.
//!
//! Rnote's text has no "raised" text attribute, so the selected characters are replaced with their Unicode
//! superscript/subscript forms (x² , H₂O, 10⁻³). Characters without such a form are kept as they are.

const SUPERSCRIPTS: &[(char, char)] = &[
    ('0', '⁰'),
    ('1', '¹'),
    ('2', '²'),
    ('3', '³'),
    ('4', '⁴'),
    ('5', '⁵'),
    ('6', '⁶'),
    ('7', '⁷'),
    ('8', '⁸'),
    ('9', '⁹'),
    ('+', '⁺'),
    ('-', '⁻'),
    ('−', '⁻'),
    ('=', '⁼'),
    ('(', '⁽'),
    (')', '⁾'),
    ('a', 'ᵃ'),
    ('b', 'ᵇ'),
    ('c', 'ᶜ'),
    ('d', 'ᵈ'),
    ('e', 'ᵉ'),
    ('f', 'ᶠ'),
    ('g', 'ᵍ'),
    ('h', 'ʰ'),
    ('i', 'ⁱ'),
    ('j', 'ʲ'),
    ('k', 'ᵏ'),
    ('l', 'ˡ'),
    ('m', 'ᵐ'),
    ('n', 'ⁿ'),
    ('o', 'ᵒ'),
    ('p', 'ᵖ'),
    ('r', 'ʳ'),
    ('s', 'ˢ'),
    ('t', 'ᵗ'),
    ('u', 'ᵘ'),
    ('v', 'ᵛ'),
    ('w', 'ʷ'),
    ('x', 'ˣ'),
    ('y', 'ʸ'),
    ('z', 'ᶻ'),
    ('A', 'ᴬ'),
    ('B', 'ᴮ'),
    ('D', 'ᴰ'),
    ('E', 'ᴱ'),
    ('G', 'ᴳ'),
    ('H', 'ᴴ'),
    ('I', 'ᴵ'),
    ('J', 'ᴶ'),
    ('K', 'ᴷ'),
    ('L', 'ᴸ'),
    ('M', 'ᴹ'),
    ('N', 'ᴺ'),
    ('O', 'ᴼ'),
    ('P', 'ᴾ'),
    ('R', 'ᴿ'),
    ('T', 'ᵀ'),
    ('U', 'ᵁ'),
    ('V', 'ⱽ'),
    ('W', 'ᵂ'),
    ('α', 'ᵅ'),
    ('β', 'ᵝ'),
    ('γ', 'ᵞ'),
    ('δ', 'ᵟ'),
    ('ε', 'ᵋ'),
    ('θ', 'ᶿ'),
    ('φ', 'ᵠ'),
    ('χ', 'ᵡ'),
    ('*', '*'),
];

const SUBSCRIPTS: &[(char, char)] = &[
    ('0', '₀'),
    ('1', '₁'),
    ('2', '₂'),
    ('3', '₃'),
    ('4', '₄'),
    ('5', '₅'),
    ('6', '₆'),
    ('7', '₇'),
    ('8', '₈'),
    ('9', '₉'),
    ('+', '₊'),
    ('-', '₋'),
    ('−', '₋'),
    ('=', '₌'),
    ('(', '₍'),
    (')', '₎'),
    ('a', 'ₐ'),
    ('e', 'ₑ'),
    ('h', 'ₕ'),
    ('i', 'ᵢ'),
    ('j', 'ⱼ'),
    ('k', 'ₖ'),
    ('l', 'ₗ'),
    ('m', 'ₘ'),
    ('n', 'ₙ'),
    ('o', 'ₒ'),
    ('p', 'ₚ'),
    ('r', 'ᵣ'),
    ('s', 'ₛ'),
    ('t', 'ₜ'),
    ('u', 'ᵤ'),
    ('v', 'ᵥ'),
    ('x', 'ₓ'),
    ('β', 'ᵦ'),
    ('γ', 'ᵧ'),
    ('ρ', 'ᵨ'),
    ('φ', 'ᵩ'),
    ('χ', 'ᵪ'),
];

fn find(table: &[(char, char)], c: char) -> Option<char> {
    table.iter().find(|(from, _)| *from == c).map(|(_, to)| *to)
}

fn find_back(table: &[(char, char)], c: char) -> Option<char> {
    table.iter().find(|(_, to)| *to == c).map(|(from, _)| *from)
}

/// Converts the text to superscript (`sup = true`) or subscript. If all convertible characters already are in that
/// form, they are converted back to normal text instead (so the button toggles).
pub fn toggle_script(text: &str, sup: bool) -> String {
    let table = if sup { SUPERSCRIPTS } else { SUBSCRIPTS };
    let already = text.chars().any(|c| find_back(table, c).is_some())
        && text
            .chars()
            .all(|c| find(table, c).is_none() || find_back(table, c).is_some());
    text.chars()
        .map(|c| {
            if already {
                find_back(table, c).unwrap_or(c)
            } else {
                // a subscript character turned into a superscript one, and the other way round
                let other = if sup { SUBSCRIPTS } else { SUPERSCRIPTS };
                let normal = find_back(other, c).unwrap_or(c);
                find(table, normal).unwrap_or(c)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superscript_and_subscript() {
        assert_eq!(toggle_script("2", true), "²");
        assert_eq!(toggle_script("-23", true), "⁻²³");
        assert_eq!(toggle_script("2", false), "₂");
        assert_eq!(toggle_script("i+1", false), "ᵢ₊₁");
        // characters without a form stay as they are
        assert_eq!(toggle_script("Q", true), "Q");
    }

    #[test]
    fn toggles_back() {
        assert_eq!(toggle_script("⁻²³", true), "-23");
        assert_eq!(toggle_script("ₙ", false), "n");
        // from subscript to superscript directly
        assert_eq!(toggle_script("₂", true), "²");
    }
}

/// OnePepe: what a date/time stamp contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeStamp {
    Date,
    Time,
    DateAndTime,
}

/// OnePepe: the text of a date/time stamp, in Spanish style (26/10/2026, 15:42).
pub fn date_time_stamp(kind: DateTimeStamp, now: chrono::NaiveDateTime) -> String {
    match kind {
        DateTimeStamp::Date => now.format("%d/%m/%Y").to_string(),
        DateTimeStamp::Time => now.format("%H:%M").to_string(),
        DateTimeStamp::DateAndTime => now.format("%d/%m/%Y %H:%M").to_string(),
    }
}

#[cfg(test)]
mod date_tests {
    use super::*;

    #[test]
    fn stamps() {
        let t = chrono::NaiveDate::from_ymd_opt(2026, 10, 5)
            .unwrap()
            .and_hms_opt(9, 7, 0)
            .unwrap();
        assert_eq!(date_time_stamp(DateTimeStamp::Date, t), "05/10/2026");
        assert_eq!(date_time_stamp(DateTimeStamp::Time, t), "09:07");
        assert_eq!(
            date_time_stamp(DateTimeStamp::DateAndTime, t),
            "05/10/2026 09:07"
        );
    }
}

/// OnePepe: list styles of a paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    /// "• "
    Bullet,
    /// "1. ", "2. "...
    Numbered,
    /// "☐ " / "☑ "
    Checkbox,
}

/// The length in bytes of the list prefix at the start of a paragraph, if any.
fn list_prefix_len(paragraph: &str) -> Option<usize> {
    for p in ["• ", "☐ ", "☑ "] {
        if paragraph.starts_with(p) {
            return Some(p.len());
        }
    }
    let digits = paragraph.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && paragraph[digits..].starts_with(". ") {
        return Some(digits + 2);
    }
    None
}

fn number_of(paragraph: &str) -> Option<u32> {
    let digits = paragraph
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    if !digits.is_empty() && paragraph[digits.len()..].starts_with(". ") {
        digits.parse().ok()
    } else {
        None
    }
}

/// The edit that applies a list style to the paragraph containing the byte index `cursor`:
/// (start of the paragraph, bytes to remove there, text to insert there).
///
/// Applying the same style again removes it, except for checkboxes, which toggle between ☐ and ☑.
pub fn list_edit(text: &str, cursor: usize, kind: ListKind) -> (usize, usize, String) {
    let cursor = cursor.min(text.len());
    let start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(text.len());
    let paragraph = &text[start..end];
    let existing = list_prefix_len(paragraph).unwrap_or(0);
    let insert = match kind {
        ListKind::Bullet => {
            if paragraph.starts_with("• ") {
                String::new()
            } else {
                "• ".to_string()
            }
        }
        ListKind::Numbered => {
            if number_of(paragraph).is_some() {
                String::new()
            } else {
                // continue the numbering of the paragraph above
                let previous = if start > 0 {
                    let prev_end = start - 1;
                    let prev_start = text[..prev_end].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    number_of(&text[prev_start..prev_end])
                } else {
                    None
                };
                format!("{}. ", previous.map(|n| n + 1).unwrap_or(1))
            }
        }
        ListKind::Checkbox => {
            if paragraph.starts_with("☐ ") {
                "☑ ".to_string()
            } else {
                "☐ ".to_string()
            }
        }
    };
    (start, existing, insert)
}

#[cfg(test)]
mod list_tests {
    use super::*;

    fn apply(text: &str, cursor: usize, kind: ListKind) -> String {
        let (start, remove, insert) = list_edit(text, cursor, kind);
        format!("{}{}{}", &text[..start], insert, &text[start + remove..])
    }

    #[test]
    fn bullets_toggle() {
        assert_eq!(apply("comprar pan", 3, ListKind::Bullet), "• comprar pan");
        assert_eq!(apply("• comprar pan", 5, ListKind::Bullet), "comprar pan");
    }

    #[test]
    fn numbering_continues() {
        let text = "1. uno\ndos";
        assert_eq!(
            apply(text, text.len(), ListKind::Numbered),
            "1. uno\n2. dos"
        );
        assert_eq!(apply("a\nb", 0, ListKind::Numbered), "1. a\nb");
        // again: removed
        assert_eq!(apply("1. uno", 4, ListKind::Numbered), "uno");
    }

    #[test]
    fn checkbox_toggles_and_replaces_other_styles() {
        assert_eq!(apply("tarea", 0, ListKind::Checkbox), "☐ tarea");
        assert_eq!(apply("☐ tarea", 5, ListKind::Checkbox), "☑ tarea");
        assert_eq!(apply("☑ tarea", 5, ListKind::Checkbox), "☐ tarea");
        assert_eq!(apply("• tarea", 5, ListKind::Checkbox), "☐ tarea");
        // only the paragraph of the cursor changes
        assert_eq!(
            apply("uno\ndos\ntres", 5, ListKind::Checkbox),
            "uno\n☐ dos\ntres"
        );
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;
    use crate::Engine;
    use crate::pens::PenStyle;
    use crate::strokes::Stroke;

    fn text_of(engine: &Engine) -> String {
        engine
            .store
            .stroke_keys_as_rendered()
            .into_iter()
            .find_map(|k| match engine.store.get_stroke_ref(k) {
                Some(Stroke::TextStroke(t)) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn list_on_the_paragraph_of_the_cursor() {
        let mut engine = Engine::default();
        let _ = engine.change_pen_style(PenStyle::Typewriter);
        let _ = engine.insert_text("1. uno\ndos".to_string(), None);
        let _ = engine.text_paragraph_list(ListKind::Numbered);
        assert_eq!(text_of(&engine), "1. uno\n2. dos");
        // the cursor stays at the end: typing continues the paragraph
        let _ = engine.insert_text(" y tres".to_string(), None);
        assert_eq!(text_of(&engine), "1. uno\n2. dos y tres");
        let _ = engine.text_paragraph_list(ListKind::Checkbox);
        assert_eq!(text_of(&engine), "1. uno\n☐ dos y tres");
        let _ = engine.text_paragraph_list(ListKind::Checkbox);
        assert_eq!(text_of(&engine), "1. uno\n☑ dos y tres");
    }

    #[test]
    fn date_stamp_creates_text() {
        let mut engine = Engine::default();
        let _ = engine.insert_date_time(DateTimeStamp::Date);
        assert_eq!(text_of(&engine).len(), "26/10/2026".len());
    }
}

/// OnePepe: what pressing Enter does at the byte index `cursor`, as an edit
/// (position, bytes to remove there, text to insert there).
///
/// Inside a list paragraph the list continues on the next line ("• ", the next number, an empty "☐ ").
/// Enter on an empty list item ends the list: its prefix is removed and no new line is added.
pub fn enter_edit(text: &str, cursor: usize) -> (usize, usize, String) {
    let cursor = cursor.min(text.len());
    let start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(text.len());
    let paragraph = &text[start..end];
    let Some(prefix_len) = list_prefix_len(paragraph) else {
        return (cursor, 0, "\n".to_string());
    };
    if cursor < start + prefix_len {
        // the cursor is inside the prefix: a plain new line
        return (cursor, 0, "\n".to_string());
    }
    if paragraph[prefix_len..].trim().is_empty() {
        return (start, prefix_len, String::new());
    }
    let next = if let Some(n) = number_of(paragraph) {
        format!("{}. ", n + 1)
    } else if paragraph.starts_with("• ") {
        "• ".to_string()
    } else {
        "☐ ".to_string()
    };
    (cursor, 0, format!("\n{next}"))
}

/// OnePepe: quick text styles, like the paragraph styles of OneNote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickStyle {
    /// Big and bold.
    Heading1,
    /// Medium and bold.
    Heading2,
    /// Italic and gray.
    Quote,
    /// Monospace.
    Code,
    /// Removes the styles.
    Normal,
}

impl std::str::FromStr for QuickStyle {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heading1" => Ok(Self::Heading1),
            "heading2" => Ok(Self::Heading2),
            "quote" => Ok(Self::Quote),
            "code" => Ok(Self::Code),
            "normal" => Ok(Self::Normal),
            _ => Err(anyhow::anyhow!("unknown quick style '{s}'")),
        }
    }
}

/// OnePepe: an editable text style of the design guide: size (and its line height) and weight.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TextStyleDef {
    pub size: f64,
    /// The line height of the design guide. Kept for the settings; the text is laid out with the line height of
    /// its font, as in Rnote (no per-paragraph line spacing in the .rnote format).
    pub line_height: f64,
    pub weight: u16,
}

impl Default for TextStyleDef {
    fn default() -> Self {
        Self {
            size: 16.0,
            line_height: 24.0,
            weight: 400,
        }
    }
}

impl TextStyleDef {
    /// The text attributes of the style.
    pub fn attributes(&self) -> Vec<crate::strokes::textstroke::TextAttribute> {
        use crate::strokes::textstroke::TextAttribute;
        vec![
            TextAttribute::FontSize(self.size),
            TextAttribute::FontWeight(self.weight),
        ]
    }
}

/// OnePepe: the text styles Title 1, Title 2 and Normal (design guide: 28/36 600, 22/30 600, 16/24 400).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TextStyles {
    pub heading1: TextStyleDef,
    pub heading2: TextStyleDef,
    pub normal: TextStyleDef,
}

impl Default for TextStyles {
    fn default() -> Self {
        Self {
            heading1: TextStyleDef {
                size: 28.0,
                line_height: 36.0,
                weight: 600,
            },
            heading2: TextStyleDef {
                size: 22.0,
                line_height: 30.0,
                weight: 600,
            },
            normal: TextStyleDef::default(),
        }
    }
}

impl TextStyles {
    /// The editable style of a quick style (Title 1, Title 2 and Normal).
    pub fn get(&self, style: QuickStyle) -> Option<TextStyleDef> {
        match style {
            QuickStyle::Heading1 => Some(self.heading1),
            QuickStyle::Heading2 => Some(self.heading2),
            QuickStyle::Normal => Some(self.normal),
            QuickStyle::Quote | QuickStyle::Code => None,
        }
    }

    pub fn set(&mut self, style: QuickStyle, def: TextStyleDef) {
        match style {
            QuickStyle::Heading1 => self.heading1 = def,
            QuickStyle::Heading2 => self.heading2 = def,
            QuickStyle::Normal => self.normal = def,
            QuickStyle::Quote | QuickStyle::Code => {}
        }
    }
}

/// The text attributes of a quick style, for text whose own size and weight are `base_size` and `base_weight`.
pub fn quick_style_attributes(
    style: QuickStyle,
    styles: &TextStyles,
    base_size: f64,
    base_weight: u16,
) -> Vec<crate::strokes::textstroke::TextAttribute> {
    use crate::strokes::textstroke::{FontStyle, TextAttribute};
    match style {
        QuickStyle::Heading1 => styles.heading1.attributes(),
        QuickStyle::Heading2 => styles.heading2.attributes(),
        QuickStyle::Quote => vec![
            TextAttribute::Style(FontStyle::Italic),
            TextAttribute::TextColor(rnote_compose::Color::new(0.4, 0.4, 0.4, 1.0)),
        ],
        QuickStyle::Code => vec![TextAttribute::FontFamily("monospace".to_string())],
        // only what differs from the text itself
        QuickStyle::Normal => styles
            .normal
            .attributes()
            .into_iter()
            .filter(|a| match a {
                TextAttribute::FontSize(size) => (size - base_size).abs() > 0.01,
                TextAttribute::FontWeight(weight) => *weight != base_weight,
                _ => true,
            })
            .collect(),
    }
}

/// OnePepe: changes the text with the style `old` to the style `new` (a text uses a style when a range of it has
/// exactly the size and the weight of the style). Returns whether something changed.
pub fn restyle_attributes(
    attributes: &mut [crate::strokes::textstroke::RangedTextAttribute],
    old: &TextStyleDef,
    new: &TextStyleDef,
) -> bool {
    use crate::strokes::textstroke::TextAttribute;
    let styled: Vec<std::ops::Range<usize>> = attributes
        .iter()
        .filter(|a| matches!(a.attribute, TextAttribute::FontSize(size) if (size - old.size).abs() < 0.01))
        .map(|a| a.range.clone())
        .filter(|range| {
            attributes.iter().any(|a| {
                a.range == *range
                    && matches!(a.attribute, TextAttribute::FontWeight(w) if w == old.weight)
            })
        })
        .collect();
    let mut changed = false;
    for a in attributes.iter_mut() {
        if !styled.contains(&a.range) {
            continue;
        }
        match &mut a.attribute {
            TextAttribute::FontSize(size) if (*size - old.size).abs() < 0.01 => {
                *size = new.size;
                changed = true;
            }
            TextAttribute::FontWeight(weight) if *weight == old.weight => {
                *weight = new.weight;
                changed = true;
            }
            _ => {}
        }
    }
    changed
}

/// The byte range of the paragraph containing the byte index `cursor`.
pub fn paragraph_range(text: &str, cursor: usize) -> std::ops::Range<usize> {
    let cursor = cursor.min(text.len());
    let start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(text.len());
    start..end
}

#[cfg(test)]
mod enter_tests {
    use super::*;

    fn apply(text: &str, cursor: usize) -> String {
        let (pos, remove, insert) = enter_edit(text, cursor);
        format!("{}{}{}", &text[..pos], insert, &text[pos + remove..])
    }

    #[test]
    fn plain_text_gets_a_new_line() {
        assert_eq!(apply("hola", 4), "hola\n");
    }

    #[test]
    fn lists_continue() {
        assert_eq!(apply("• uno", "• uno".len()), "• uno\n• ");
        assert_eq!(apply("1. uno", 6), "1. uno\n2. ");
        assert_eq!(apply("9. x", 4), "9. x\n10. ");
        assert_eq!(apply("☑ hecho", "☑ hecho".len()), "☑ hecho\n☐ ");
    }

    #[test]
    fn empty_item_ends_the_list() {
        let text = "• uno\n• ";
        assert_eq!(apply(text, text.len()), "• uno\n");
        let text = "1. a\n2. ";
        assert_eq!(apply(text, text.len()), "1. a\n");
    }

    #[test]
    fn quick_styles() {
        let styles = TextStyles::default();
        assert_eq!(
            quick_style_attributes(QuickStyle::Heading1, &styles, 20.0, 500).len(),
            2
        );
        assert!(quick_style_attributes(QuickStyle::Normal, &styles, 16.0, 400).is_empty());
        assert_eq!(
            quick_style_attributes(QuickStyle::Normal, &styles, 20.0, 400).len(),
            1
        );
        assert_eq!(
            "heading2".parse::<QuickStyle>().unwrap(),
            QuickStyle::Heading2
        );
        assert_eq!(paragraph_range("a\nbcd\ne", 3), 2..5);
        assert_eq!(paragraph_range("abc", 1), 0..3);
    }
}

#[cfg(test)]
mod restyle_tests {
    use super::*;
    use crate::strokes::textstroke::{RangedTextAttribute, TextAttribute};

    #[test]
    fn restyle_changes_only_the_text_with_the_style() {
        let styles = TextStyles::default();
        let ranged =
            |range: std::ops::Range<usize>, attribute| RangedTextAttribute { range, attribute };
        let mut attributes = vec![
            ranged(0..5, TextAttribute::FontSize(28.0)),
            ranged(0..5, TextAttribute::FontWeight(600)),
            // same size, other weight: not Title 1
            ranged(6..9, TextAttribute::FontSize(28.0)),
            ranged(6..9, TextAttribute::FontWeight(400)),
        ];
        let new = TextStyleDef {
            size: 32.0,
            line_height: 40.0,
            weight: 700,
        };
        assert!(restyle_attributes(&mut attributes, &styles.heading1, &new));
        assert!(matches!(attributes[0].attribute, TextAttribute::FontSize(s) if s == 32.0));
        assert!(matches!(
            attributes[1].attribute,
            TextAttribute::FontWeight(700)
        ));
        assert!(matches!(attributes[2].attribute, TextAttribute::FontSize(s) if s == 28.0));
        assert!(!restyle_attributes(&mut attributes, &styles.heading2, &new));
    }
}

#[cfg(test)]
mod quick_style_engine_tests {
    use crate::Engine;
    use crate::pens::PenStyle;
    use crate::strokes::Stroke;
    use crate::strokes::textstroke::TextAttribute;

    #[test]
    fn heading_keeps_size_and_weight() {
        let mut engine = Engine::default();
        let _ = engine.change_pen_style(PenStyle::Typewriter);
        let _ = engine.insert_text("Apuntes".to_string(), None);
        let _ = engine.text_quick_style(super::QuickStyle::Heading1, &super::TextStyles::default());
        let key = engine.store.stroke_keys_as_rendered()[0];
        let Some(Stroke::TextStroke(textstroke)) = engine.store.get_stroke_ref(key) else {
            panic!("no text stroke");
        };
        let attrs = &textstroke.text_style.ranged_text_attributes;
        assert!(
            attrs
                .iter()
                .any(|a| matches!(a.attribute, TextAttribute::FontSize(_)))
        );
        assert!(
            attrs
                .iter()
                .any(|a| matches!(a.attribute, TextAttribute::FontWeight(600)))
        );
        assert!(attrs.iter().all(|a| a.range == (0..7)));
    }
}
