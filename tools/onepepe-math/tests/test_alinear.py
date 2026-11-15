# SPDX-License-Identifier: GPL-3.0-or-later
"""Alineado ordenado, reparto por hash y estados de la medida (ARQUITECTURA-EJECUCION.md §6 M0 y T0)."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import numpy as np  # noqa: E402

from onepepe_math import alinear  # noqa: E402
from onepepe_math.entreno import expected_symbols, list_sets, load_set  # noqa: E402

import medir  # noqa: E402


def trazos(simbolos):
    """Trazos de una lista [(etiqueta, nº de trazos)] en orden de escritura. La coordenada y de cada trazo es el
    número de su símbolo; los trazos de un símbolo se solapan en x (así quedan «encadenados»)."""
    return [[(20.0 * s, float(s)), (20.0 * s + 10, float(s))] for s, (_, k) in enumerate(simbolos) for _ in range(k)]


class Plantilla:
    """Clasificador falso: grupo = símbolo entero -> 0,9 de su etiqueta; trazo plano suelto (de «=» o «+») -> 0,45
    de «-» y 0,05 de «=»; cualquier otro grupo -> 0,9 de «trozo»."""

    def __init__(self, simbolos, clases=None):
        self.simbolos = simbolos
        etiquetas = clases if clases is not None else sorted({e for e, _ in simbolos})
        self.classes = list(etiquetas) + ["-", "=", "__trozo__"]
        self.classes = list(dict.fromkeys(self.classes))

    def probabilities(self, strokes):
        ys = [int(s[0][1]) for s in strokes]
        p = np.full(len(self.classes), 0.1 / len(self.classes))
        sim = set(ys)
        if len(sim) == 1:
            etiqueta, k = self.simbolos[ys[0]]
            if len(ys) == k and etiqueta in self.classes:
                p[self.classes.index(etiqueta)] = 0.9
                return p
            if len(ys) == 1 and etiqueta in ("=", "+"):
                p[self.classes.index("-")] = 0.45
                p[self.classes.index("=")] = 0.05
                p[self.classes.index("__trozo__")] = 0.4
                return p
        p[self.classes.index("__trozo__")] = 0.9
        return p


def alinear_(simbolos, latex, **kw):
    tokens = expected_symbols(latex)
    return alinear.ordered_groups(trazos(simbolos), tokens, Plantilla(simbolos), latex=latex, **kw)


def tamanos(res):
    return [len(g) for g, _ in res["grupos"]]


def etiquetas(res):
    return [e for _, e in res["grupos"]]


class TestAlineado(unittest.TestCase):
    def test_orden_respeta_fichas(self):
        r = alinear_([("2", 1), ("x", 2), ("+", 2), ("1", 1), ("=", 2), ("7", 2)], "2x+1=7")
        self.assertEqual(tamanos(r), [1, 2, 2, 1, 2, 2])
        self.assertEqual(etiquetas(r), ["2", "x", "+", "1", "=", "7"])

    def test_frac_raya_en_medio(self):
        r = alinear_([("1", 1), (r"\frac", 1), ("2", 1)], r"\frac{1}{2}")
        self.assertEqual(etiquetas(r), ["1", r"\frac", "2"])

    def test_frac_raya_antes_y_despues(self):
        r = alinear_([(r"\frac", 1), ("1", 1), ("2", 1)], r"\frac{1}{2}")
        self.assertEqual(etiquetas(r), [r"\frac", "1", "2"])
        r = alinear_([("1", 1), ("2", 1), (r"\frac", 1)], r"\frac{1}{2}")
        self.assertEqual(etiquetas(r), ["1", "2", r"\frac"])

    def test_acento_despues(self):
        r = alinear_([("y", 2), (r"\hat", 1)], r"\hat{y}")
        self.assertEqual(etiquetas(r), ["y", r"\hat"])

    def test_limites_al_reves(self):
        r = alinear_([(r"\sum", 1), ("3", 1), ("i", 2), ("=", 2), ("1", 1)], r"\sum_{i=1}^{3}")
        self.assertIsNotNone(r)
        e = etiquetas(r)
        self.assertLess(e.index("3"), e.index("i"))

    def test_tardio(self):
        sim = [("i", 2), ("t", 2)]
        t = trazos(sim)  # i, i, t, t -> el punto de la «i» y la raya de la «t» se escriben al final
        t = [t[0], t[2], t[1], t[3]]
        clf = Plantilla(sim)
        sin = alinear.ordered_groups(t, ["i", "t"], clf, latex="it")
        self.assertNotEqual(sorted(map(sorted, (g for g, _ in sin["grupos"]))), [[0, 2], [1, 3]])
        con = alinear.ordered_groups(t, ["i", "t"], clf, latex="it", tardios=True)
        self.assertEqual(sorted(map(sorted, (g for g, _ in con["grupos"]))), [[0, 2], [1, 3]])
        self.assertIsNotNone(con["movido"])

    def test_imposible(self):
        self.assertIsNone(alinear.ordered_groups(trazos([("1", 1), ("2", 1)]), ["1", "2", "3"], Plantilla([("1", 1), ("2", 1)])))

    def test_ficha_desconocida(self):
        sim = [("2", 1), ("Z", 1)]
        r = alinear.ordered_groups(trazos(sim), ["2", "Z"], Plantilla(sim, clases=["2"]), latex="2Z")
        self.assertIsNotNone(r)
        self.assertEqual(r["desconocido"], [False, True])

    def test_muchas_variantes(self):
        sim = [("1", 1), (r"\frac", 1), ("2", 1)] * 5
        latex = r"\frac{1}{2}" * 5
        self.assertEqual(len(alinear._estructuras(latex, expected_symbols(latex))[1]), 5)
        r = alinear_(sim, latex)
        self.assertIsNotNone(r)
        self.assertEqual(len(r["grupos"]), 15)

    def test_arbol_no_cuadra(self):
        latex = r"\foo{1}2"
        v = list(alinear.variantes_de_orden(latex, expected_symbols(latex)))
        self.assertEqual(v, [[0, 1, 2]])
        self.assertIsNotNone(alinear_([(r"\foo", 1), ("1", 1), ("2", 1)], latex))
        # y si el árbol no tiene las mismas fichas que los símbolos, también una sola
        self.assertEqual(list(alinear.variantes_de_orden(r"\frac{1}{2}", ["a"])), [[0]])

    def test_variantes_de_frac(self):
        v = list(alinear.variantes_de_orden(r"\frac{1}{2}", expected_symbols(r"\frac{1}{2}")))
        self.assertEqual(v, [[1, 0, 2], [0, 1, 2], [1, 2, 0]])


class TestReparto(unittest.TestCase):
    def test_hash_fijo(self):
        self.assertAlmostEqual(alinear.u("onepepe-examen-v1", "general-200#g001"), 0.562747, places=6)
        self.assertAlmostEqual(alinear.u("onepepe-examen-v1", "asignaturas-200#a001"), 0.581660, places=6)
        self.assertAlmostEqual(alinear.u("onepepe-validacion-v1", "general-200#g001"), 0.866588, places=6)

    def sets(self):
        return {s["name"]: [(i["id"], i["kind"]) for i in load_set(s["name"])["items"]] for s in list_sets()}

    def test_reparto(self):
        sets = self.sets()
        r = alinear.reparto(sets)
        for s in sets:
            self.assertEqual(len(r["examen"][s]), 15 if s == "refuerzo" else 40)
            self.assertFalse(set(r["examen"][s]) & set(r["validacion"][s]))
        mas = dict(sets, nuevo=[(f"n{i}", "expr") for i in range(30)])
        r2 = alinear.reparto(mas)
        for s in sets:
            self.assertEqual(r2["examen"][s], r["examen"][s])
            self.assertEqual(r2["validacion"][s], r["validacion"][s])
        self.assertIn(alinear.pliegue("general-200#g001"), range(4))


class TestEstados(unittest.TestCase):
    def test_estados(self):
        refs = [[0], [1, 2]]
        self.assertEqual(medir.estados(refs, [[0, 1], [2]]), ["juntado", "mixto"])
        self.assertEqual(medir.estados(refs, [[0], [1], [2]]), ["exacto", "partido"])


if __name__ == "__main__":
    unittest.main()
