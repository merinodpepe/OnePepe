# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests del programa auxiliar y del reconocedor completo."""

import io
import json
import os
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from onepepe_math import glyphs  # noqa: E402
from onepepe_math.classifier import DEFAULT_MODEL  # noqa: E402
from onepepe_math.server import Server  # noqa: E402


class StubClassifier:
    """Clasificador de pega: 1 trazo -> "1", 2 trazos -> "=" ; sin modelo de contexto."""

    lm = None

    def candidates_w_junk(self, strokes, k=5):
        if len(strokes) == 1:
            return [("1", 0.99), ("l", 0.01)], 0.0
        if len(strokes) == 2:
            return [("=", 0.99), ("-", 0.01)], 0.0
        return [("x", 0.01)], 0.99

    def embedding(self, strokes):
        # formas distintas para cada trazo: no se agrupan
        v = np.zeros(128)
        v[int(strokes[0][0][0]) % 128] = 1.0
        return v


class StubPersonal:
    def __len__(self):
        return 0


def line(x0, y0, x1, y1, n=10):
    return [[x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n] for i in range(n + 1)]


class TestServer(unittest.TestCase):
    def run_server(self, *requests):
        server = Server(StubClassifier(), StubPersonal())
        out = io.StringIO()
        server.serve(io.StringIO("\n".join(json.dumps(r) for r in requests) + "\n"), out)
        return [json.loads(x) for x in out.getvalue().splitlines()]

    def test_protocol(self):
        # "1 + 1 =" escrito como: 1, (dos rayas cruzadas: no hace falta), 1, =
        strokes = [
            line(0, 0, 0, 20),
            line(30, 0, 30, 20),
            line(50, 7, 64, 7),
            line(50, 13, 64, 13),
        ]
        resp = self.run_server(
            {"id": 1, "cmd": "ping"},
            {"id": 2, "cmd": "recognize", "strokes": strokes, "threshold": 0.9},
            {"id": 3, "cmd": "nada"},
            {"id": 4, "cmd": "glyph", "latex": "K", "box": [0, 0, 10, 20]},
        )
        self.assertEqual([r["id"] for r in resp], [1, 2, 3, 4])
        self.assertTrue(resp[0]["ok"])
        r = resp[1]
        self.assertEqual([s["candidates"][0]["latex"] for s in r["symbols"]], ["1", "1", "="])
        self.assertEqual(r["latex"], "11=")
        self.assertEqual(r["result"]["text"], "11")
        self.assertIsNotNone(r["symbols"][0]["clean"])
        self.assertIn("error", resp[2])
        self.assertTrue(resp[3]["clean"])

    def test_recognize_group_and_segment_fix(self):
        from onepepe_math.personal import PersonalModel

        with tempfile.TemporaryDirectory() as d:
            personal = PersonalModel(StubClassifier(), os.path.join(d, "c.jsonl"))
            server = Server(StubClassifier(), personal)
            two = [line(50, 7, 64, 7), line(50, 13, 64, 13)]
            r = server.handle({"cmd": "recognize_group", "strokes": two})
            self.assertEqual(r["candidates"][0]["latex"], "=")
            self.assertTrue(r["clean"])
            r = server.handle({"cmd": "segment_fix", "junk": [[two[0]], [two[1]]]})
            self.assertEqual(r["saved"], 2)
            # los "trozos" se guardan para reentrenar, pero no entran en el kNN
            self.assertEqual(personal.count("junk"), 2)
            self.assertEqual(len(personal), 0)

    def test_bad_json_does_not_crash(self):
        server = Server(StubClassifier(), StubPersonal())
        out = io.StringIO()
        server.serve(io.StringIO("esto no es json\n" + json.dumps({"id": 7, "cmd": "ping"}) + "\n"), out)
        lines = [json.loads(x) for x in out.getvalue().splitlines()]
        self.assertIn("error", lines[0])
        self.assertEqual(lines[1]["id"], 7)


class TestGlyphs(unittest.TestCase):
    def test_all_core_symbols_with_glyph_fit_in_box(self):
        for latex in glyphs.supported_symbols():
            lines = glyphs.place_glyph(latex, (100, 50, 120, 80))
            self.assertTrue(lines, latex)
            xs = [p[0] for ln in lines for p in ln]
            ys = [p[1] for ln in lines for p in ln]
            # centrado en la caja y sin salirse mucho
            self.assertAlmostEqual((min(xs) + max(xs)) / 2, 110, delta=0.5, msg=latex)
            self.assertAlmostEqual((min(ys) + max(ys)) / 2, 65, delta=0.5, msg=latex)
            self.assertLessEqual(max(ys) - min(ys), 30.01, latex)

    def test_y_axis_points_down(self):
        # el "7" tiene la raya arriba: sus puntos más altos (y menor) forman una línea horizontal
        lines = glyphs.place_glyph("7", (0, 0, 10, 20))
        top = min(p[1] for ln in lines for p in ln)
        top_pts = [p for ln in lines for p in ln if abs(p[1] - top) < 0.5]
        self.assertGreater(max(p[0] for p in top_pts) - min(p[0] for p in top_pts), 5)


@unittest.skipUnless(os.path.exists(DEFAULT_MODEL), "falta el modelo (train_classifier.py)")
class TestWithModel(unittest.TestCase):
    def test_personal_correction_is_learned(self):
        from onepepe_math.classifier import SymbolClassifier
        from onepepe_math.features import augment
        from onepepe_math.personal import PersonalModel

        clf = SymbolClassifier()
        # una "K" propia: tres trazos raros que el modelo general no lee como K
        k = [line(0, 0, 0, 30), line(0, 15, 14, 0), line(4, 11, 16, 30)]
        rng = np.random.default_rng(0)
        with tempfile.TemporaryDirectory() as d:
            personal = PersonalModel(clf, os.path.join(d, "c.jsonl"))
            for _ in range(3):
                personal.add_correction("K", augment(k, rng))
            # se guarda en disco y se vuelve a cargar
            self.assertEqual(len(PersonalModel(clf, os.path.join(d, "c.jsonl"))), 3)
            sample = augment(k, rng)
            adjusted = personal.adjust(sample, clf.candidates(sample, 5))
            self.assertEqual(adjusted[0][0], "K")


if __name__ == "__main__":
    unittest.main()
