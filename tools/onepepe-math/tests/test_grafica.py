# SPDX-License-Identifier: GPL-3.0-or-later
"""Gráficas de funciones: x², 1/x, tan x y √x (intervalo, cortes y puntos)."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from test_pagina import row  # noqa: E402

from onepepe_math.grafica import plot_rows  # noqa: E402


def squared(text, y=0.0):
    out = row(text, y)
    for s in out["symbols"]:
        if s["latex"] == "2":
            s["box"] = [s["box"][0], y + 2, s["box"][0] + 5, y + 9]
    return out


class TestPlot(unittest.TestCase):
    def test_parabola(self):
        r = plot_rows([squared("y=x2")], size=(300, 200))
        self.assertEqual(r["x_range"], [-10.0, 10.0])
        self.assertEqual(len(r["curves"]), 1)
        self.assertEqual(len(r["curves"][0]), 1)  # una sola línea, sin cortes
        pts = r["curves"][0][0]
        # dentro de la caja y con el mínimo en el centro, abajo
        self.assertTrue(all(0 <= x <= 300 and 0 <= y <= 200 for x, y in pts))
        lowest = max(pts, key=lambda p: p[1])
        self.assertAlmostEqual(lowest[0], 150, delta=3)
        self.assertTrue(r["axes"] and r["labels"])

    def test_one_over_x_is_cut_at_zero(self):
        r = plot_rows([row("y=1/x")])
        self.assertEqual(len(r["curves"][0]), 2)

    def test_tan_has_several_branches(self):
        r = plot_rows([row("y=tan(x)")])
        self.assertGreaterEqual(len(r["curves"][0]), 6)

    def test_sqrt_only_where_it_exists(self):
        rows = [{"symbols": [{"latex": "y", "box": [0, 10, 8, 26]}, {"latex": "=", "box": [10, 12, 18, 17]},
                             {"latex": r"\sqrt", "box": [20, 2, 40, 22]}, {"latex": "x", "box": [28, 10, 36, 20]}]}]
        r = plot_rows(rows)
        xs = [x for line in r["curves"][0] for x, _ in line]
        self.assertGreaterEqual(min(xs), 149)

    def test_range_row_and_function_notation(self):
        rng = row(["x", r"\in", "[", "0", ";", "5", "]"], 40)
        r = plot_rows([row("f(x)=x+1"), rng])
        self.assertEqual(r["x_range"], [0.0, 5.0])
        rng = row(["x", r"\in", "[", "0", ",", "5", "]"], 40)
        self.assertEqual(plot_rows([row("f(x)=x+1"), rng])["x_range"], [0.0, 5.0])

    def test_two_curves_and_nothing_to_plot(self):
        r = plot_rows([row("y=x"), row("y=2x", 40)])
        self.assertEqual(len(r["curves"]), 2)
        self.assertIn("error", plot_rows([row("2+3")]))


if __name__ == "__main__":
    unittest.main()
