# SPDX-License-Identifier: GPL-3.0-or-later
"""Modo entreno: el set de 200, dibujar las expresiones e importar lo escrito (con un clasificador falso)."""

import glob
import json
import os
import shutil
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math import entreno  # noqa: E402
from onepepe_math.symbols import CORE  # noqa: E402
from test_alinear import Plantilla, trazos  # noqa: E402

try:
    import matplotlib  # noqa: F401

    HAVE_MPL = True
except ImportError:
    HAVE_MPL = False


class TestSet(unittest.TestCase):
    def sets(self):
        return [entreno.load_set(s["name"]) for s in entreno.list_sets()]

    def test_two_hundred_with_unique_ids(self):
        names = [d["name"] for d in self.sets()]
        self.assertIn("general-200", names)
        self.assertIn("asignaturas-200", names)
        all_ids = []
        for data in self.sets():
            items = data["items"]
            self.assertEqual(len(items), 200, data["name"])
            all_ids += [it["id"] for it in items]
            self.assertEqual({it["kind"] for it in items}, {"expr", "steps", "symbols"})
            self.assertTrue(all(len(it["lines"]) >= 2 for it in items if it["kind"] == "steps"))
        # ids únicos también entre sets (las respuestas se guardan por id)
        self.assertEqual(len(all_ids), len(set(all_ids)))

    def test_every_symbol_is_known_by_the_recognizer(self):
        for data in self.sets():
            for it in data["items"]:
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
        for data in self.sets():
            for it in data["items"]:
                self.assertIsNotNone(entreno.render(tuple(it["lines"])), it["lines"])

    def test_listed(self):
        self.assertIn("general-200", [s["name"] for s in entreno.list_sets()])


class FakeStore:
    def __init__(self, path):
        self.path = path
        self.items = []

    def add_correction(self, latex, strokes, notebook=None, kind="symbol", **_):
        self.items.append((latex, len(strokes), kind, notebook))


def sesion(latex, simbolos, id_="g001", set_="general-200"):
    """Una sesión con una expresión escrita con los trazos de `trazos(simbolos)` (ver test_alinear)."""
    return {"set": set_, "items": [{"id": id_, "kind": "expr", "lines": [latex]}],
            "answers": {id_: {"strokes": trazos(simbolos), "size": [300, 100]}}}


IGUAL = [("2", 1), ("x", 1), ("+", 1), ("1", 1), ("=", 2), ("7", 1)]


class TestImport(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.dir)
        self.store = FakeStore(os.path.join(self.dir, "correcciones.jsonl"))

    def test_import_usa_orden(self):
        out = entreno.import_session(sesion("2x+1=7", IGUAL), Plantilla(IGUAL), self.store, examen=set())
        self.assertEqual(out["imported"], ["g001"])
        self.assertEqual([x[1] for x in self.store.items if x[0] == "="], [2])

    def test_import_salta_examen(self):
        out = entreno.import_session(sesion("2x+1=7", IGUAL), Plantilla(IGUAL), self.store, examen={"g001"})
        self.assertEqual(self.store.items, [])
        self.assertEqual((out["imported"], out["examen"]), (["g001"], ["g001"]))

    def test_import_revision(self):
        sim = [("x", 1), ("+", 1), ("1", 1)]
        out = entreno.import_session(sesion("x+1", sim), Plantilla(sim, clases=["x"]), self.store, examen=set())
        self.assertEqual([x[0] for x in self.store.items if x[2] == "symbol"], ["x"])
        self.assertEqual(out["revision"], 2)
        with open(os.path.join(self.dir, "revision.jsonl"), encoding="utf-8") as f:
            self.assertEqual([json.loads(l)["latex"] for l in f], ["+", "1"])

    def test_notebook_lleva_id(self):
        entreno.import_session(sesion("2x+1=7", IGUAL), Plantilla(IGUAL), self.store, examen=set())
        self.assertTrue(self.store.items)
        self.assertTrue(all(x[3] == "entreno:general-200#g001" for x in self.store.items))

    def test_forzar_ignora_la_marca(self):
        s = sesion("2x+1=7", IGUAL)
        s["answers"]["g001"]["imported"] = True
        clf = Plantilla(IGUAL)
        self.assertEqual(entreno.import_session(s, clf, self.store, examen=set())["imported"], [])
        self.assertEqual(entreno.import_session(s, clf, self.store, examen=set(), forzar=True)["imported"], ["g001"])


class ClasificadorEmbedding(Plantilla):
    version = "t"

    def embedding(self, strokes):
        return np.ones(4)


class TestReimportar(unittest.TestCase):
    def test_reimportar_copia(self):
        import reimportar_entrenos
        from onepepe_math.personal import PersonalModel

        d = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, d)
        ruta = os.path.join(d, "correcciones.jsonl")
        clf = ClasificadorEmbedding(IGUAL)
        pm = PersonalModel(clf, ruta, background=False)
        pm.add_correction("x", trazos([("x", 1)]), notebook="entreno:general-200#g009")
        pm.add_correction("y", trazos([("y", 1)]), notebook="mi_pagina.rnote")
        ses = os.path.join(d, "ses")
        os.mkdir(ses)
        with open(os.path.join(ses, "a.entreno.json"), "w", encoding="utf-8") as f:
            json.dump(sesion("2x+1=7", IGUAL), f)
        # ensayo: nada cambia
        t = reimportar_entrenos.reimportar(ses, clf, ruta)
        self.assertEqual(t["entran"], 6)
        self.assertEqual(len(PersonalModel(clf, ruta, background=False).examples), 2)
        t = reimportar_entrenos.reimportar(ses, clf, ruta, aplicar=True)
        self.assertEqual(len(glob.glob(os.path.join(d, "correcciones.*.bak"))), 1)
        notas = [e["notebook"] for e in PersonalModel(clf, ruta, background=False).examples.values()]
        self.assertIn("mi_pagina.rnote", notas)
        self.assertNotIn("entreno:general-200#g009", notas)
        self.assertEqual(notas.count("entreno:general-200#g001"), t["entran"] + t["trozos"])


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


class TestTextWithoutMatplotlib(unittest.TestCase):
    def test_readable_text(self):
        self.assertEqual(entreno.to_unicode(r"3+4\cdot 5="), "3 + 4 · 5 =")
        self.assertEqual(entreno.to_unicode(r"x^2+y^2=r^2"), "x² + y² = r²")
        self.assertEqual(entreno.to_unicode(r"\hat{\beta}=(X^\top X)^{-1}X^\top y"), "β̂ = (Xᵀ X)⁻¹Xᵀ y")
        self.assertEqual(entreno.to_unicode(r"\frac{1}{2}+\frac{1}{3}="), "1/2 + 1/3 =")
        self.assertEqual(entreno.to_unicode(r"-3+(-5)="), "−3 + (−5) =")
        self.assertEqual(entreno.to_unicode(r"\mathbb{E}[\bar{x}]"), "𝔼[x̄]")

    def test_answer_always_has_text(self):
        r = entreno.render_answer([r"2x+4=10", r"x=3"])
        self.assertEqual(r["text"], "2x + 4 = 10\nx = 3")
        # every expression of both sets has a readable text
        for s in entreno.list_sets():
            for it in entreno.load_set(s["name"])["items"]:
                for line in it["lines"]:
                    self.assertTrue(entreno.to_unicode(line), line)
