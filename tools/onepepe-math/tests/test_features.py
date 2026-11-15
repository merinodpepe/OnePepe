# SPDX-License-Identifier: GPL-3.0-or-later
"""Rasgos v2 (ARQUITECTURA-EJECUCION.md M7.2-M7.5): invariancias del histograma de direcciones y contexto del renglón."""

import math
import os
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math import features as F  # noqa: E402

DIR = slice(160, 288)  # bloque de direcciones
CTX = slice(544, 550)  # bloque de contexto


def recta(grados, largo=10.0, x0=0.0, y0=0.0):
    a = math.radians(grados)
    return [(x0, y0), (x0 + largo * math.cos(a), y0 + largo * math.sin(a))]


A = [[(0, 0), (10, 20), (20, 0)], [(5, 10), (15, 10)]]


class TestFeatures(unittest.TestCase):
    def test_v1_intacto(self):
        v = F.symbol_features_v1(A)
        self.assertEqual(v.shape, (266,))
        self.assertAlmostEqual(float(v.sum()), 50.0435, places=3)
        self.assertAlmostEqual(float((v * np.arange(len(v))).sum()), 8447.495, places=1)
        np.testing.assert_allclose(v[:6], [-1.0, -1.0, 0.44721, 0.89443, 0.0, -0.91326], atol=1e-4)
        np.testing.assert_allclose(v[-6:], [0.0, 0.0, 1.0, 0.0, 0.0, 1 / 3], atol=1e-4)

    def test_tamano_vector(self):
        self.assertEqual(F.symbol_features(A).shape, (556,))
        self.assertEqual(F.symbol_features(A, version=1).shape, (266,))

    def test_orden_no_importa(self):
        a, b = A
        np.testing.assert_allclose(F.symbol_features([a, b])[DIR], F.symbol_features([b, a])[DIR], atol=1e-6)

    def test_sentido_no_importa(self):
        s = [(0, 0), (8, 3), (12, 12), (3, 20)]
        np.testing.assert_allclose(F.symbol_features([s])[DIR], F.symbol_features([s[::-1]])[DIR], atol=1e-6)

    def test_continuidad(self):
        d = np.abs(F.symbol_features([recta(22.4)])[DIR] - F.symbol_features([recta(22.6)])[DIR]).max()
        self.assertLess(d, 0.1)

    def test_contexto_distingue_tamano(self):
        forma = [[(0, 0), (6, 0), (6, 10), (0, 10), (0, 0)]]
        lin = (10.0, 0.0, 10.0)
        grande = F.symbol_features(forma, lin)
        chico = F.symbol_features([[(x * 0.6, y * 0.6 + 4) for x, y in forma[0]]], lin)
        self.assertNotAlmostEqual(float(grande[CTX][2]), float(chico[CTX][2]))
        np.testing.assert_allclose(np.delete(grande, np.r_[CTX]), np.delete(chico, np.r_[CTX]), atol=1e-3)

    def test_sin_contexto(self):
        self.assertTrue((F.symbol_features(A, None)[CTX] == 0).all())
        self.assertEqual(F.symbol_features(A, (10.0, 0.0, 10.0))[CTX][0], 1.0)

    def test_modelo_viejo_funciona(self):
        from onepepe_math.classifier import JUNK, SymbolClassifier
        from onepepe_math.symbols import CORE

        rng = np.random.default_rng(0)
        clases = ["1", "x", JUNK]
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "viejo.npz")
            np.savez(p, classes=np.array(clases), mean=np.zeros(266, np.float32), std=np.ones(266, np.float32),
                     n_layers=np.array(2), W0=rng.normal(size=(266, 8)).astype(np.float32),
                     b0=np.zeros(8, np.float32), W1=rng.normal(size=(8, 3)).astype(np.float32),
                     b1=np.zeros(3, np.float32))
            clf = SymbolClassifier(p, vocabulary=CORE)
            self.assertFalse(clf.usa_linea)
            self.assertAlmostEqual(float(clf.probabilities(A, linea=(10.0, 0.0, 5.0)).sum()), 1.0, places=5)
            self.assertEqual(clf.embedding(A).shape, (8,))

    def test_linea_base_robusta(self):
        # cinco símbolos de alto 10 con fondo en y=10 y una «g» que baja hasta 18
        t = [[(20.0 * i, 0.0), (20.0 * i + 8, 10.0)] for i in range(5)] + [[(100.0, 2.0), (108.0, 18.0)]]
        tip, pend, corte = F.linea(t)
        self.assertLess(abs(corte - 10.0), 0.05 * tip)
        self.assertIsNone(F.linea(t[:1]))

    def test_featurizar_v2(self):
        import train_classifier as tc

        rng = np.random.default_rng(0)
        m = [("A", A, "a")]
        X, y, _ = tc.featurizar(m, [2], rng, [(10.0, 0.0, 10.0)], 2, soltar=0.0)
        self.assertEqual((X.shape, y), ((3, 556), ["A"] * 3))
        self.assertTrue((X[:, CTX][:, 0] == 1).all())
        X, _, _ = tc.featurizar(m, [2], rng, [(10.0, 0.0, 10.0)], 2, soltar=1.0)
        self.assertTrue((X[:, CTX] == 0).all())
        self.assertEqual(tc.featurizar(m, [1], rng)[0].shape, (2, 266))


if __name__ == "__main__":
    unittest.main()
