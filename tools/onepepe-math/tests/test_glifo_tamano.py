# SPDX-License-Identifier: GPL-3.0-or-later
"""El símbolo limpio ocupa el alto de la tinta (∫ ∑ ∏ ( 1) y los planos no cambian."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from onepepe_math import glyphs  # noqa: E402


class TamanoGlifo(unittest.TestCase):
    def test_simbolos_altos_ocupan_el_alto(self):
        for s in (r"\int", r"\sum", r"\prod", "(", "1"):
            x0, y0, x1, y1 = glyphs._bounds(glyphs.place_glyph(s, (0, 0, 10, 40)))
            self.assertGreaterEqual(y1 - y0, 0.95 * 40, s)
            self.assertAlmostEqual((y0 + y1) / 2, 20, delta=2, msg=s)

    def test_simbolo_ancho_no_cambia(self):
        x0, y0, x1, y1 = glyphs._bounds(glyphs.place_glyph("=", (0, 0, 40, 10)))
        self.assertAlmostEqual(x1 - x0, 40, places=1)
        self.assertAlmostEqual(y1 - y0, 13.33, places=1)


if __name__ == "__main__":
    unittest.main()
