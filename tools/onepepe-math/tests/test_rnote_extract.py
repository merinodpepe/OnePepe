# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests del extractor. Ejecutar desde tools/onepepe-math:  python3 -m unittest discover -s tests"""

import json
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

import rnote_extract  # noqa: E402

FIXTURE = os.path.join(HERE, "fixtures", "dos_mas_tres.rnote")


class TestExtract(unittest.TestCase):
    def test_fixture(self):
        # "2+3=" dibujado con el pincel del motor de Rnote: 6 trazos, un renglón
        version, snap = rnote_extract.load_rnote(FIXTURE)
        self.assertEqual(version, "0.15.0")
        strokes = rnote_extract.extract_strokes(snap)
        self.assertEqual(len(strokes), 6)
        # orden de escritura: el "2" (x = 100..122) es el primero, el "=" (x = 220..244) el último
        self.assertLess(strokes[0]["bbox"][0], 110)
        self.assertGreater(strokes[-1]["bbox"][0], 215)
        for s in strokes:
            self.assertGreater(len(s["points"]), 5)
            self.assertEqual(len(s["points"][0]), 3)
        lines = rnote_extract.split_lines(strokes)
        self.assertEqual(len(lines), 1)
        self.assertEqual(lines[0]["stroke_ids"], [0, 1, 2, 3, 4, 5])

    def test_split_lines(self):
        def stroke(i, y):
            return {"id": i, "order": i, "points": [], "bbox": [0, y, 20, y + 30]}

        # dos expresiones separadas por un hueco grande; el segundo renglón tiene un trazo algo más bajo
        strokes = [stroke(0, 0), stroke(1, 5), stroke(2, 200), stroke(3, 215)]
        lines = rnote_extract.split_lines(strokes)
        self.assertEqual([line["stroke_ids"] for line in lines], [[0, 1], [2, 3]])

    def test_cli_with_labels_and_svg(self):
        with tempfile.TemporaryDirectory() as d:
            labels = os.path.join(d, "labels.txt")
            with open(labels, "w", encoding="utf-8") as f:
                f.write("# comentario\n2+3=\n")
            out = os.path.join(d, "out.json")
            rnote_extract.main([FIXTURE, "-o", out, "--svg", os.path.join(d, "svg"), "--labels", labels])
            with open(out, encoding="utf-8") as f:
                data = json.load(f)
            self.assertEqual(data["lines"][0]["latex"], "2+3=")
            self.assertTrue(os.path.exists(os.path.join(d, "svg", "renglon_00.svg")))


if __name__ == "__main__":
    unittest.main()
