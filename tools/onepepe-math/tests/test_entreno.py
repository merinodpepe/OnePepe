# SPDX-License-Identifier: GPL-3.0-or-later
"""Modo entreno: el set de 200, dibujar las expresiones e importar lo escrito (con un clasificador falso)."""

import os
import sys
import unittest

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math import entreno  # noqa: E402
from onepepe_math.symbols import CORE  # noqa: E402

try:
    import matplotlib  # noqa: F401

    HAVE_MPL = True
except ImportError:
    HAVE_MPL = False


class TestSet(unittest.TestCase):
    def setUp(self):
        self.data = entreno.load_set("general-200")

    def test_two_hundred_with_unique_ids(self):
        items = self.data["items"]
        self.assertEqual(len(items), 200)
        self.assertEqual(len({it["id"] for it in items}), 200)
        kinds = {it["kind"] for it in items}
        self.assertEqual(kinds, {"expr", "steps", "symbols"})
        self.assertTrue(all(len(it["lines"]) >= 2 for it in items if it["kind"] == "steps"))

    def test_every_symbol_is_known_by_the_recognizer(self):
        for it in self.data["items"]:
            for latex in it["lines"]:
                symbols = entreno.expected_symbols(latex)
                self.assertTrue(symbols, latex)
                unknown = [s for s in symbols if s not in CORE]
                self.assertEqual(unknown, [], f"{it['id']}: {latex}")

    def test_expected_symbols(self):
        self.assertEqual(entreno.expected_symbols(r"\sin(0)="), ["s", "i", "n", "(", "0", ")", "="])
        self.assertEqual(entreno.expected_symbols(r"\mathrm{sen}\,x"), ["s", "e", "n", "x"])
        self.assertEqual(entreno.expected_symbols(r"\frac{1}{2}"), [r"\frac", "1", "2"])
        self.assertEqual(entreno.expected_symbols(r"i\quad i"), ["i", "i"])

    @unittest.skipUnless(HAVE_MPL, "sin matplotlib")
    def test_every_expression_can_be_drawn(self):
        for it in self.data["items"]:
            for latex in it["lines"]:
                self.assertIsNotNone(entreno.render(latex), latex)

    def test_listed(self):
        self.assertIn("general-200", [s["name"] for s in entreno.list_sets()])


class FakeClassifier:
    """Cualquier trazo suelto es cualquiera de los símbolos (lo que cuenta aquí es el reparto, no la red)."""

    def __init__(self, labels):
        self.classes = sorted(set(labels))

    def probabilities(self, strokes):
        p = np.full(len(self.classes), 1.0 / len(self.classes))
        return p if len(strokes) == 1 else p * 0.01


class FakeStore:
    def __init__(self):
        self.items = []

    def add_correction(self, latex, strokes, notebook=None, kind="symbol", **_):
        self.items.append((latex, len(strokes), kind, notebook))


def dash(x, y):
    return [[x, y], [x + 5, y + 8]]


class TestImport(unittest.TestCase):
    def session(self):
        return {
            "set": "prueba",
            "items": [
                {"id": "a", "kind": "expr", "lines": ["2x+1=7"]},
                {"id": "b", "kind": "steps", "lines": ["x=1", "y=2"]},
                {"id": "c", "kind": "expr", "lines": ["x+1"]},
                {"id": "d", "kind": "expr", "lines": ["9"]},
            ],
            "answers": {
                # seis trazos, un símbolo cada uno
                "a": {"strokes": [dash(10 * i, 10) for i in range(6)], "size": [300, 100]},
                # dos renglones guía: cada trazo va al renglón de su altura
                "b": {"strokes": [dash(0, 10), dash(10, 10), dash(20, 10), dash(0, 60), dash(10, 60), dash(20, 60)],
                      "size": [300, 100]},
                # un trazo para tres símbolos: no encaja
                "c": {"strokes": [dash(0, 10)], "size": [300, 100]},
                "d": {"strokes": [dash(0, 10)], "size": [300, 100], "skipped": True},
            },
        }

    def test_import(self):
        labels = list("2x+1=7y9")
        store = FakeStore()
        out = entreno.import_session(self.session(), FakeClassifier(labels), store)
        self.assertEqual(sorted(out["imported"]), ["a", "b"])
        self.assertEqual(out["failed"], ["c"])
        self.assertEqual(out["symbols"], 12)
        self.assertEqual(sorted(x[0] for x in store.items if x[2] == "symbol")[:3], ["+", "1", "1"])
        self.assertTrue(all(x[3] == "entreno:prueba" for x in store.items))
        # lo ya importado no se repite
        s = self.session()
        for k in ("a", "b"):
            s["answers"][k]["imported"] = True
        self.assertEqual(entreno.import_session(s, FakeClassifier(labels), FakeStore())["imported"], [])


class TestServer(unittest.TestCase):
    def test_commands(self):
        from onepepe_math.server import Server

        server = Server(classifier=object(), personal=[])
        session = server.handle({"cmd": "training_new", "set": "general-200"})["session"]
        self.assertEqual(session["format"], "onepepe-entreno")
        self.assertEqual(len(session["items"]), 200)
        self.assertEqual(session["answers"], {})
        r = server.handle({"cmd": "training_render", "lines": [r"\frac{1}{2}", "x=1"], "color": "#ffffff"})
        if HAVE_MPL:
            self.assertTrue(r["png"])
        self.assertIn("sets", server.handle({"cmd": "training_sets"}))


if __name__ == "__main__":
    unittest.main()
