# SPDX-License-Identifier: GPL-3.0-or-later
"""Corte en símbolos: parámetros, "trozo", pausas y corte forzado, con un clasificador falso."""

import os
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math.forced import forced_groups, junk_examples  # noqa: E402
from onepepe_math.segment import SegmentParams, segment  # noqa: E402


class FakeClassifier:
    """Dos rayas horizontales juntas = "=", una sola raya = "-" (pero con mucho de "trozo")."""

    classes = ["-", "=", "1", "__trozo__"]

    def probabilities(self, strokes):
        if len(strokes) == 2 and all(abs(s[0][1] - s[-1][1]) < 1 for s in strokes):
            return np.array([0.05, 0.9, 0.04, 0.01])
        if len(strokes) == 1 and abs(strokes[0][0][1] - strokes[0][-1][1]) < 1:
            return np.array([0.5, 0.05, 0.05, 0.4])
        if len(strokes) == 1:
            return np.array([0.05, 0.05, 0.85, 0.05])
        return np.array([0.1, 0.1, 0.1, 0.7])

    def candidates_w_junk(self, strokes, k=5):
        p = self.probabilities(strokes)
        idx = [i for i in np.argsort(-p) if self.classes[i] != "__trozo__"][:k]
        return [(self.classes[i], float(p[i])) for i in idx], float(p[3])


def hline(y, x0=0.0, x1=10.0):
    return [(x0, y), (x1, y)]


def vline(x, y0=0.0, y1=10.0):
    return [(x, y0), (x, y1)]


class TestSegment(unittest.TestCase):
    def test_params_roundtrip_and_defaults(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "corte.json")
            self.assertEqual(SegmentParams.load(path), SegmentParams())
            SegmentParams(touch_gap=0.4).save(path)
            self.assertEqual(SegmentParams.load(path).touch_gap, 0.4)

    def test_equals_sign_is_one_symbol(self):
        # "1 =" : la raya suelta tiene mucho de "trozo", así que las dos rayas van juntas
        strokes = [vline(0), hline(3, 20, 30), hline(6, 20, 30)]
        syms = segment(strokes, FakeClassifier())
        self.assertEqual([s["strokes"] for s in syms], [[0], [1, 2]])
        self.assertEqual(syms[1]["candidates"][0][0], "=")

    def test_pauses_only_count_when_known(self):
        strokes = [vline(0), hline(3, 20, 30), hline(6, 20, 30)]
        params = SegmentParams(pause_weight=2.0)
        same = segment(strokes, FakeClassifier(), params=params)
        # una pausa larga entre las dos rayas las separa
        times = [(0.0, 0.2), (1.0, 1.2), (3.5, 3.7)]
        split = segment(strokes, FakeClassifier(), params=params, times=times)
        self.assertEqual(len(same), 2)
        self.assertEqual(len(split), 3)

    def test_forced_groups_and_junk(self):
        strokes = [vline(0), hline(3, 20, 30), hline(6, 20, 30)]
        labeled = forced_groups(strokes, ["1", "="], FakeClassifier())
        self.assertEqual(labeled, [([0], "1"), ([1, 2], "=")])
        junk = junk_examples(strokes, labeled)
        # las dos rayas sueltas del "=" y el "1" pegado al "="
        self.assertEqual(len(junk), 3)
        self.assertIsNone(forced_groups(strokes, ["1", "=", "-", "-"], FakeClassifier()))


if __name__ == "__main__":
    unittest.main()
