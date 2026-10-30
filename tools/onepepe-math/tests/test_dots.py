# SPDX-License-Identifier: GPL-3.0-or-later
"""La i y la j (PLAN-PENDIENTE.md §2, punto 4), con un clasificador falso."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math.dots import fix_dotted_letters  # noqa: E402


def box_of(strokes):
    xs = [p[0] for s in strokes for p in s]
    ys = [p[1] for s in strokes for p in s]
    return min(xs), min(ys), max(xs), max(ys)


class FakeClassifier:
    """Un palo con un punto encima: «i» (o «j» casi igual de probable)."""

    def candidates_w_junk(self, strokes, k=5):
        if len(strokes) == 2:
            return [("i", 0.5), ("j", 0.4), (";", 0.1)], 0.0
        return [("1", 0.9)], 0.0


def stem(x, y0, y1):
    return [(x, y0), (x, y1)]


def dot(x, y):
    return [(x, y), (x + 0.5, y + 0.5)]


class TestDots(unittest.TestCase):
    def test_dot_cut_apart_is_joined_and_baseline_decides(self):
        # «a i» con la i en la línea base (y hacia abajo; base en y=20)
        strokes = [[(0, 10), (8, 20)], stem(14, 12, 20), dot(14, 7)]
        syms = [
            {"strokes": [0], "candidates": [("a", 0.9)], "junk": 0.0},
            {"strokes": [1], "candidates": [("1", 0.9)], "junk": 0.0},
            {"strokes": [2], "candidates": [(".", 0.9)], "junk": 0.0},
        ]
        out = fix_dotted_letters(syms, strokes, FakeClassifier(), typical=10.0)
        self.assertEqual([s["strokes"] for s in out], [[0], [1, 2]])
        self.assertEqual(out[1]["candidates"][0][0], "i")
        self.assertGreaterEqual(out[1]["candidates"][0][1], 0.9)

    def test_j_goes_below_the_baseline(self):
        strokes = [[(0, 10), (8, 20)], stem(14, 12, 26), dot(14, 7)]
        syms = [
            {"strokes": [0], "candidates": [("a", 0.9)], "junk": 0.0},
            {"strokes": [1], "candidates": [("1", 0.9)], "junk": 0.0},
            {"strokes": [2], "candidates": [(".", 0.9)], "junk": 0.0},
        ]
        out = fix_dotted_letters(syms, strokes, FakeClassifier(), typical=10.0)
        self.assertEqual(out[1]["candidates"][0][0], "j")

    def test_known_symbols_give_the_baseline(self):
        strokes = [stem(14, 12, 26), dot(14, 7)]
        syms = [{"strokes": [0, 1], "candidates": [("i", 0.6), ("j", 0.3)], "junk": 0.0}]
        known = [{"latex": "x", "box": [0, 10, 8, 20]}]
        out = fix_dotted_letters(syms, strokes, FakeClassifier(), typical=10.0, known=known)
        self.assertEqual(out[0]["candidates"][0][0], "j")
        # sin nada con que comparar, se queda lo que diga la red
        out = fix_dotted_letters(syms, strokes, FakeClassifier(), typical=10.0)
        self.assertEqual(out[0]["candidates"][0][0], "i")

    def test_a_decimal_point_is_not_joined(self):
        # «1.5»: el punto está abajo, al lado, no encima
        strokes = [stem(0, 10, 20), dot(4, 19.5), [(8, 10), (14, 20)]]
        syms = [
            {"strokes": [0], "candidates": [("1", 0.9)], "junk": 0.0},
            {"strokes": [1], "candidates": [(".", 0.9)], "junk": 0.0},
            {"strokes": [2], "candidates": [("5", 0.9)], "junk": 0.0},
        ]
        out = fix_dotted_letters(syms, strokes, FakeClassifier(), typical=10.0)
        self.assertEqual(len(out), 3)


if __name__ == "__main__":
    unittest.main()
