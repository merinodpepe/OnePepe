# SPDX-License-Identifier: GPL-3.0-or-later
"""Extracción de símbolos de MathWriting (ARQUITECTURA-EJECUCION.md M7.7): filtro, criterio del 80 % y reanudación."""

import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import extraer_simbolos as ex  # noqa: E402
from test_alinear import Plantilla, trazos  # noqa: E402

SIMBOLOS = [("2", 1), ("x", 2), ("+", 2), ("1", 1), ("=", 2), ("7", 2)]
CORE = {"2", "x", "+", "1", "=", "7"}


def formulas(n):
    return [(f"f{i:02d}", "2x+1=7", trazos(SIMBOLOS)) for i in range(n)]


class TestExtraer(unittest.TestCase):
    def test_filtro(self):
        self.assertTrue(ex.formula_valida(["2", "x"], 3, CORE))
        self.assertFalse(ex.formula_valida(["2", "z"], 3, CORE))  # ficha fuera de CORE
        self.assertFalse(ex.formula_valida(["2"] * 26, 40, {"2"}))  # más de 25 fichas
        self.assertFalse(ex.formula_valida(["2", "x"], 9, CORE))  # n > 4m
        self.assertFalse(ex.formula_valida(["2", "x"], 1, CORE))  # n < m

    def test_criterio_80(self):
        self.assertTrue(ex.supera_80([True] * 4 + [False]))
        self.assertFalse(ex.supera_80([True] * 3 + [False] * 2))

    def test_linea(self):
        t = [[(20.0 * i, 0.0), (20.0 * i + 10, 10.0)] for i in range(5)]
        tip, pend, corte = ex.linea(t)
        self.assertAlmostEqual(pend, 0.0)
        self.assertAlmostEqual(corte, 10.0)
        self.assertIsNone(ex.linea(t[:1]))

    def test_extrae_y_reanuda(self):
        with tempfile.TemporaryDirectory() as d:
            sal = os.path.join(d, "s.jsonl")
            clf = Plantilla(SIMBOLOS)
            # parada por clase: con 2 de cada clase bastan 2 fórmulas
            r = ex.extraer(formulas(6), clf, sal, core=CORE, por_clase=2, cada=1)
            self.assertEqual(r["vistas"], 2)
            self.assertEqual(r["validas"], 2)
            with open(sal) as f:
                lineas = f.readlines()
            filas = [json.loads(x) for x in lineas]
            self.assertEqual(len(filas), 12)
            self.assertEqual({x["id"] for x in filas}, {"f00", "f01"})
            self.assertEqual(filas[0].keys(), {"etiqueta", "trazos", "id", "linea"})
            # reanudar desde «una fórmula vista», con restos de la segunda en el archivo: se cortan y no hay duplicados
            with open(sal + ".progreso") as f:
                p = json.load(f)
            p.update(vistas=1, validas=1, bytes=len("".join(lineas[:6]).encode()))
            p["conteo"] = {}
            for x in filas[:6]:
                p["conteo"][x["etiqueta"]] = p["conteo"].get(x["etiqueta"], 0) + 1
            with open(sal + ".progreso", "w") as f:
                json.dump(p, f)
            r = ex.extraer(formulas(6), clf, sal, core=CORE, por_clase=2, cada=1)
            self.assertEqual(r["vistas"], 2)
            with open(sal) as f:
                ids = [json.loads(x)["id"] for x in f]
            self.assertEqual(ids, ["f00"] * 6 + ["f01"] * 6)

    def test_tope_de_formulas(self):
        with tempfile.TemporaryDirectory() as d:
            r = ex.extraer(formulas(6), Plantilla(SIMBOLOS), os.path.join(d, "s.jsonl"), core=CORE,
                           por_clase=99, tope=3, cada=2)
            self.assertEqual(r["vistas"], 3)


if __name__ == "__main__":
    unittest.main()
