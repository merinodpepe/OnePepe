//! OnePepe: paleta de colores predefinida, ampliada (como la de OneNote).
//!
//! Filas: grises y luego cinco tonos (muy claro, claro, medio, oscuro, muy oscuro).
//! Columnas: rojo, naranja, amarillo, verde, verde azulado, azul, morado, rosa.

// Imports
use crate::Color;

/// Número de columnas de la paleta.
pub const PALETTE_COLUMNS: usize = 8;

/// Colores de la paleta en hexadecimal RGB, por filas.
const PALETTE_HEX: [[u32; PALETTE_COLUMNS]; 6] = [
    // grises, de blanco a negro
    [
        0xffffff, 0xe5e5e5, 0xa3a3a3, 0x737373, 0x525252, 0x404040, 0x262626, 0x000000,
    ],
    // muy claros
    [
        0xfecaca, 0xfed7aa, 0xfef08a, 0xbbf7d0, 0x99f6e4, 0xbfdbfe, 0xe9d5ff, 0xfbcfe8,
    ],
    // claros
    [
        0xf87171, 0xfb923c, 0xfacc15, 0x4ade80, 0x2dd4bf, 0x60a5fa, 0xc084fc, 0xf472b6,
    ],
    // medios
    [
        0xdc2626, 0xea580c, 0xca8a04, 0x16a34a, 0x0d9488, 0x2563eb, 0x9333ea, 0xdb2777,
    ],
    // oscuros
    [
        0x991b1b, 0x9a3412, 0x854d0e, 0x166534, 0x115e59, 0x1e40af, 0x6b21a8, 0x9d174d,
    ],
    // muy oscuros
    [
        0x450a0a, 0x431407, 0x422006, 0x052e16, 0x042f2e, 0x172554, 0x3b0764, 0x500724,
    ],
];

fn color_from_hex(hex: u32) -> Color {
    Color::new(
        f64::from((hex >> 16) & 0xff) / 255.0,
        f64::from((hex >> 8) & 0xff) / 255.0,
        f64::from(hex & 0xff) / 255.0,
        1.0,
    )
}

/// Todos los colores de la paleta, por filas (de izquierda a derecha y de arriba abajo).
pub fn palette_colors() -> Vec<Color> {
    PALETTE_HEX
        .iter()
        .flat_map(|row| row.iter().map(|&hex| color_from_hex(hex)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_has_at_least_24_unique_colors() {
        let colors = palette_colors();
        assert!(colors.len() >= 24);
        assert_eq!(colors.len() % PALETTE_COLUMNS, 0);
        for (i, a) in colors.iter().enumerate() {
            for b in colors.iter().skip(i + 1) {
                assert!(!a.approx_eq(*b), "color repetido: {a:?}");
            }
        }
    }

    #[test]
    fn palette_has_light_and_dark_colors() {
        let colors = palette_colors();
        let light = colors.iter().filter(|c| c.luma() > 0.8).count();
        let dark = colors.iter().filter(|c| c.luma() < 0.15).count();
        assert!(light >= 6, "pocos colores claros: {light}");
        assert!(dark >= 6, "pocos colores oscuros: {dark}");
    }

    #[test]
    fn hex_conversion() {
        let c = color_from_hex(0xff8000);
        assert!(c.approx_eq(Color::new(1.0, 128.0 / 255.0, 0.0, 1.0)));
    }
}
