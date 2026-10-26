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
