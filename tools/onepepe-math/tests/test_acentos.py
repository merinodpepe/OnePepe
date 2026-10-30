# SPDX-License-Identifier: GPL-3.0-or-later
"""Acentos de estadística (ŷ, x̄, x̃), sin romper fracciones ni «=»."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from test_pagina import row  # noqa: E402

from onepepe_math import entreno, nombres, pagina  # noqa: E402
from onepepe_math.evaluate import answer  # noqa: E402
from onepepe_math.glyphs import place_glyph  # noqa: E402
from onepepe_math.layout import Sym, parse, to_latex  # noqa: E402


def x(box_x=0.0, label="x"):
    return Sym(label, (box_x, 10, box_x + 8, 20))


class TestAccents(unittest.TestCase):
    def test_hat_and_bar(self):
        y_hat = [Sym("y", (0, 10, 8, 26)), Sym(r"\hat", (1, 3, 7, 7))]
        self.assertEqual(to_latex(parse(y_hat)), r"\hat{y}")
        # la barra de la media se lee «-» y es un acento si es corta y no hay nada encima
        x_bar = [x(), Sym("-", (0, 6, 8, 7))]
        self.assertEqual(to_latex(parse(x_bar)), r"\overline{x}")
        # con subíndice
        y_hat_i = y_hat + [Sym("i", (9, 20, 12, 28))]
        self.assertEqual(to_latex(parse(y_hat_i)), r"\hat{y}_{i}")

    def test_fraction_and_equals_are_not_accents(self):
        frac = [Sym("1", (2, 0, 6, 8)), Sym("-", (0, 10, 8, 11)), Sym("2", (2, 13, 6, 21))]
        self.assertEqual(to_latex(parse(frac)), r"\frac{1}{2}")
        eq = [x(), Sym("=", (10, 12, 18, 17)), Sym("3", (20, 4, 28, 20))]
        self.assertEqual(to_latex(parse(eq)), "x=3")
        # una raya larga encima de varios símbolos no es la media de uno
        long_bar = [x(0), x(10, "y"), Sym("-", (0, 6, 18, 7))]
        self.assertNotIn(r"\overline", to_latex(parse(long_bar)))

    def test_accented_symbols_are_variables_of_their_own(self):
        x_bar = [x(), Sym("-", (0, 6, 8, 7))]
        # x̄ - x̄ = 0, y x̄ no es x
        syms = x_bar + [Sym("-", (12, 14, 18, 15))] + [Sym(s.label, (s.box[0] + 22, *s.box[1:2], s.box[2] + 22, s.box[3]))
                                                       for s in x_bar] + [Sym("=", (34, 12, 42, 17))]
        self.assertEqual(answer(parse(syms))["text"], "0")
        # x̄ = 5 define x̄ en la página, y no x
        r = pagina.page([{"symbols": [{"latex": "x", "box": [0, 10, 8, 20]}, {"latex": "-", "box": [0, 6, 8, 7]},
                                      {"latex": "=", "box": [10, 12, 18, 17]}, {"latex": "5", "box": [20, 4, 28, 20]}]},
                         {"symbols": [{"latex": "x", "box": [0, 50, 8, 60]}, {"latex": "-", "box": [0, 46, 8, 47]},
                                      {"latex": "+", "box": [10, 51, 18, 58]}, {"latex": "1", "box": [20, 44, 28, 60]}]},
                         row("x+1", 80)], variables=True)
        self.assertEqual([m["text"] for m in r[1]["marks"]], ["= 6"])
        self.assertEqual(r[2]["marks"], [])

    def test_clean_glyphs_names_and_training(self):
        for s in (r"\hat", r"\overline", r"\tilde", r"\rightleftharpoons", r"\langle", r"\rangle", r"\top"):
            self.assertTrue(place_glyph(s, (0, 0, 10, 4)), s)
        self.assertEqual(nombres.resolve("media"), r"\overline")
        self.assertEqual(nombres.resolve("esperanza"), r"\mathbb{E}")
        self.assertEqual(entreno.expected_symbols(r"\mathbb{E}[\bar{x}]"), [r"\mathbb{E}", "[", r"\overline", "x", "]"])


if __name__ == "__main__":
    unittest.main()
