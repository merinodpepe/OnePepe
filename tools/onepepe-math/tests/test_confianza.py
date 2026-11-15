# SPDX-License-Identifier: GPL-3.0-or-later
"""M8: cota de Wilson, regla de sustitución (red + kNN + umbral) y marcas dudosas."""

import os
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from test_pagina import row  # noqa: E402

from calibrar_umbral import calibrar  # noqa: E402
from onepepe_math import confianza, pagina  # noqa: E402
from onepepe_math.recognizer import recognize  # noqa: E402


class Red:
    lm = None

    def __init__(self, label="x", p=0.99):
        self.label, self.p = label, p

    def candidates_w_junk(self, strokes, k=5):
        return [(self.label, self.p), ("z", 1 - self.p)], 0.0

    def embedding(self, strokes):
        return np.ones(4)


class Personal:
    """Un kNN de pega: `vecino` a distancia 0.05 y la mezcla que devuelva `final`."""

    def __init__(self, vecino=None, final=None):
        self.vecino, self.final = vecino, final

    def __len__(self):
        return 1

    def neighbors(self, e, k=5):
        return [(0.05, self.vecino, 1)] if self.vecino else []

    def adjust(self, g, cands, embedding=None):
        return self.final or cands


TRAZO = [[(0.0, 0.0), (10.0, 10.0)]]


def lee(red, personal=None, **kw):
    return recognize(TRAZO, red, personal=personal, equals=False, **kw)["symbols"][0]


class TestConfianza(unittest.TestCase):
    def test_wilson(self):
        for k, n, esperado in ((0, 100, 0.0162), (0, 300, 0.0054), (5, 100, 0.0859)):
            self.assertAlmostEqual(confianza.wilson_cs(k, n), esperado, delta=0.0005)

    def test_no_sustituye_si_knn_contradice(self):
        s = lee(Red("x", 0.99), Personal(vecino="y"))
        self.assertIsNone(s["clean"])
        self.assertFalse(s["sure"])

    def test_sustituye_misma_forma(self):
        s = lee(Red("x", 0.99), Personal(final=[("X", 0.99), ("z", 0.01)]))
        self.assertTrue(s["sure"])
        self.assertIsNotNone(s["clean"])

    def test_no_sustituye_si_la_red_no_coincide(self):
        s = lee(Red("x", 0.99), Personal(final=[("z", 0.99), ("x", 0.01)]))
        self.assertFalse(s["sure"])

    def test_umbral_es_el_mayor(self):
        self.assertFalse(lee(Red("x", 0.96), threshold=0.95, umbral_min=0.98)["sure"])
        self.assertTrue(lee(Red("x", 0.96), threshold=0.95, umbral_min=0.90)["sure"])

    def test_confianza_json(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertEqual(confianza.cargar(d), {})
            with open(os.path.join(d, "confianza.json"), "w") as f:
                f.write('{"umbral": 0.98}')
            self.assertEqual(confianza.cargar(d)["umbral"], 0.98)

    def test_marca_dudosa(self):
        r = row("1+2=")
        r["symbols"][0]["p"] = 0.4
        self.assertFalse(pagina.page([r], umbral_renglon=0.7)[0]["marks"][0]["sure"])
        self.assertTrue(pagina.page([r], umbral_renglon=0.3)[0]["marks"][0]["sure"])

    def test_sin_p_es_seguro(self):
        self.assertTrue(pagina.page([row("1+2=")])[0]["marks"][0]["sure"])

    def test_calibrar_elige_el_menor_que_cumple(self):
        # 400 símbolos: a partir de 0,98 ninguno falla; por debajo, el 10 % falla
        sim = [[0.99, True, True]] * 400 + [[0.92, True, i % 10 > 0] for i in range(100)]
        res = calibrar(sim, [[0.9, True], [0.4, False]])
        self.assertEqual(res["umbral"], 0.95)
        self.assertTrue(res["cumple"])
        self.assertEqual(res["umbral_renglon"], 0.5)


if __name__ == "__main__":
    unittest.main()
