# SPDX-License-Identifier: GPL-3.0-or-later
"""Reparación general del corte (M5.4) y papel de la raya (M5.5), con un clasificador falso."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math.layout import Sym, bar_role  # noqa: E402
from onepepe_math.reparar import reparar  # noqa: E402


class FakeClassifier:
    """Dos rayas = «=» (muy seguro); una raya sola, «-» con dudas; con el «1» no hay ganancia."""

    def candidates_w_junk(self, strokes, k=5):
        if len(strokes) == 2 and all(abs(s[0][1] - s[-1][1]) < 1 for s in strokes):
            return [("=", 0.95)], 0.0
        if len(strokes) == 1 and abs(strokes[0][0][1] - strokes[0][-1][1]) < 1:
            return [("-", 0.6)], 0.3
        if len(strokes) == 1:
            return [("1", 0.95)], 0.0
        return [("1", 0.1)], 0.8


def hline(y, x0=0.0, x1=10.0):
    return [(x0, y), (x1, y)]


def sym(i, label, p=0.9):
    return {"strokes": [i], "candidates": [(label, p)], "junk": 0.0}


class TestReparar(unittest.TestCase):
    def test_repara_igual_partido(self):
        # dos «-» apilados con un «1» escrito entre medias
        strokes = [hline(0), [(30, -5), (30, 5)], hline(4)]
        syms = [sym(0, "-", 0.6), sym(1, "1"), sym(2, "-", 0.6)]
        out = reparar(syms, strokes, FakeClassifier(), typical=10.0)
        self.assertEqual([s["strokes"] for s in out], [[0, 2], [1]])
        self.assertEqual(out[0]["candidates"][0][0], "=")

    def test_no_repara_sin_ganancia(self):
        strokes = [hline(0), [(5, 3), (5, 13)]]
        syms = [sym(0, "-", 0.6), sym(1, "1")]
        out = reparar(syms, strokes, FakeClassifier(), typical=10.0)
        self.assertEqual([s["strokes"] for s in out], [[0], [1]])


class TestPapelRaya(unittest.TestCase):
    def test_papel_raya(self):
        raya = (0, 10, 30, 10)
        con = [Sym("1", (10, 0, 14, 8)), Sym("2", (10, 12, 14, 20))]
        self.assertEqual(bar_role(raya, con, 10.0), "frac")
        self.assertEqual(bar_role(raya, [], 10.0), "minus")
        self.assertEqual(bar_role((8, 8, 16, 8.2), [Sym("x", (9, 9, 15, 19))], 10.0), "overline")


if __name__ == "__main__":
    unittest.main()
