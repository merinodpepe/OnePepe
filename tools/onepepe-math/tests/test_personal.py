# SPDX-License-Identifier: GPL-3.0-or-later
"""Almacén de correcciones y kNN, con un clasificador falso (no hace falta el modelo)."""

import json
import os
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math.personal import D_MAX, NeighborIndex, PersonalModel  # noqa: E402


class FakeClassifier:
    """El embedding es la dirección del primer trazo: formas iguales, embeddings iguales."""

    def __init__(self, version="v1"):
        self.version = version
        self.calls = 0

    def embedding(self, strokes):
        self.calls += 1
        (x0, y0), (x1, y1) = strokes[0][0], strokes[0][-1]
        return np.array([x1 - x0, y1 - y0, 0.1], dtype=np.float32)


def stroke(dx, dy):
    return [[(0.0, 0.0), (dx, dy)]]


class TestPersonal(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.path = os.path.join(self.dir.name, "c.jsonl")

    def tearDown(self):
        self.dir.cleanup()

    def test_empty_store_changes_nothing(self):
        m = PersonalModel(FakeClassifier(), self.path)
        cands = [("x", 0.7), ("\\times", 0.3)]
        self.assertEqual(m.adjust(stroke(1, 1), cands), cands)

    def test_close_neighbor_moves_the_vote(self):
        m = PersonalModel(FakeClassifier(), self.path)
        m.add_correction("x", stroke(1, 1), predicted="\\times", notebook="/cuaderno")
        out = m.adjust(stroke(1, 1), [("\\times", 0.6), ("x", 0.4)])
        self.assertEqual(out[0][0], "x")
        self.assertAlmostEqual(sum(p for _, p in out), 1.0)

    def test_far_neighbor_is_ignored(self):
        m = PersonalModel(FakeClassifier(), self.path)
        m.add_correction("x", stroke(1, 0))
        cands = [("1", 0.9), ("l", 0.1)]
        # perpendicular: distancia ~1 > D_MAX
        self.assertGreater(1.0, D_MAX)
        self.assertEqual(m.adjust(stroke(0, 1), cands), cands)

    def test_record_fields_forget_and_relabel(self):
        m = PersonalModel(FakeClassifier(), self.path)
        i = m.add_correction("K", stroke(1, 2), predicted="k", notebook="/biofisica")
        with open(self.path, encoding="utf-8") as f:
            item = json.loads(f.readline())
        for key in ("id", "latex", "predicted", "strokes", "embedding", "model", "date", "notebook"):
            self.assertIn(key, item)
        self.assertTrue(m.relabel(i, "\\kappa"))
        self.assertEqual(PersonalModel(FakeClassifier(), self.path).list_examples()[0]["latex"], "\\kappa")
        self.assertTrue(m.forget(i))
        self.assertEqual(len(PersonalModel(FakeClassifier(), self.path)), 0)

    def test_new_model_version_recomputes_embeddings(self):
        PersonalModel(FakeClassifier("v1"), self.path).add_correction("x", stroke(1, 1))
        clf = FakeClassifier("v2")
        m = PersonalModel(clf, self.path, background=False)
        self.assertEqual(clf.calls, 1)
        self.assertEqual(len(m), 1)
        with open(self.path, encoding="utf-8") as f:
            self.assertEqual(json.loads(f.readline())["model"], "v2")

    def test_old_format_is_read(self):
        with open(self.path, "w", encoding="utf-8") as f:
            f.write(json.dumps({"latex": "x", "strokes": stroke(1, 1)}) + "\n")
        self.assertEqual(len(PersonalModel(FakeClassifier(), self.path, background=False)), 1)

    def test_index(self):
        idx = NeighborIndex()
        self.assertEqual(idx.neighbors(np.ones(3), 3), [])
        idx.rebuild([("a", np.array([1.0, 0, 0])), ("b", np.array([0, 1.0, 0]))])
        d, i = idx.neighbors(np.array([1.0, 0.1, 0]), 1)[0]
        self.assertEqual(i, "a")
        self.assertLess(d, 0.01)


if __name__ == "__main__":
    unittest.main()
