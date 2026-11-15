# SPDX-License-Identifier: GPL-3.0-or-later
"""Entrenamiento sin fugas (ARQUITECTURA-EJECUCION.md §6 M2, T2): reparto, tope por clase, temperatura, examen fuera."""

import os
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import train_classifier as tc  # noqa: E402
from onepepe_math.classifier import SymbolClassifier  # noqa: E402
from onepepe_math.features import N_FEATURES  # noqa: E402

try:
    from sklearn.neural_network import MLPClassifier
except ImportError:  # pragma: no cover
    MLPClassifier = None

RAYA = [[(0.0, 0.0), (10.0, 5.0)]]


class TestEntrenamiento(unittest.TestCase):
    @unittest.skipUnless(MLPClassifier, "sin scikit-learn")
    def test_partial_fit_una_epoca(self):
        rng = np.random.default_rng(0)
        X, y = rng.normal(size=(60, 4)), rng.integers(0, 3, 60)
        clf = MLPClassifier(hidden_layer_sizes=(8,), batch_size=16, random_state=0)
        for _ in range(3):
            clf.partial_fit(X, y, classes=np.arange(3))
        # n_iter_ vale 1 en cada llamada (no acumula); una pasada por llamada se ve en t_ y en loss_curve_
        self.assertEqual((clf.n_iter_, clf.t_, len(clf.loss_curve_)), (1, 180, 3))

    def test_reparto_antes_de_deformar(self):
        muestras = [("x", RAYA, "0" * 10 + f"{(7 * i) % 256:02x}00") for i in range(20)]
        val = [m for m in muestras if tc.es_validacion_mw(m[2])]
        train = [m for m in muestras if not tc.es_validacion_mw(m[2])]
        self.assertTrue(val and train)
        rng = np.random.default_rng(0)
        _, _, o_tr = tc.featurizar(train, [5] * len(train), rng)
        _, _, o_va = tc.featurizar(val, [0] * len(val), rng)
        self.assertEqual(len(o_tr), 6 * len(train))
        self.assertFalse(set(o_tr) & set(o_va))

    def test_tope_por_clase(self):
        self.assertLessEqual(100 * tc.copias_propias(100, 30), 40 * 30)
        self.assertEqual(tc.copias_propias(40, 30), 30)
        self.assertEqual(tc.copias_propias(2, 90), 90)
        self.assertEqual(tc.copias_trozo(100000, 1000), 15)

    def test_temperatura_newton(self):
        rng = np.random.default_rng(3)
        z0 = rng.normal(0, 2, size=(5000, 6))
        p = np.exp(z0 - z0.max(axis=1, keepdims=True))
        p /= p.sum(axis=1, keepdims=True)
        y = np.array([rng.choice(6, p=q) for q in p])
        self.assertTrue(1.8 < tc.ajustar_temperatura(2 * z0, y) < 2.2)

    def test_temperatura_no_cambia_orden(self):
        rng = np.random.default_rng(1)
        with tempfile.TemporaryDirectory() as d:
            ruta = os.path.join(d, "m.npz")
            base = dict(classes=np.array(["0", "1", "2"]), mean=np.zeros(N_FEATURES), std=np.ones(N_FEATURES),
                        n_layers=np.array(1), W0=rng.normal(size=(N_FEATURES, 3)).astype(np.float32),
                        b0=np.zeros(3, np.float32))
            np.savez(ruta, **base)
            uno = SymbolClassifier(ruta)
            np.savez(ruta, **base, temperature=np.array(3.0))
            tres = SymbolClassifier(ruta)
        a, b = uno.candidates(RAYA), tres.candidates(RAYA)
        self.assertEqual([x for x, _ in a], [x for x, _ in b])
        self.assertGreater(a[0][1], b[0][1])

    def test_examen_fuera(self):
        def it(nb, latex="x"):
            return {"latex": latex, "strokes": RAYA, "notebook": nb, "id": nb}

        conj = {"examen": {"general-200": ["g001"]}, "validacion": {"general-200": ["g002"]}}
        items = [it("entreno:general-200#g001"), it("entreno:general-200#g002"), it("entreno:general-200#g003"),
                 it("mi_pagina.rnote")]
        train, val = tc.reparto_propio(items, conj)
        self.assertEqual([r[2] for r in val], ["general-200#g002"])
        self.assertEqual(sorted(r[2] for r in train), ["general-200#g003", "mi_pagina.rnote"])


if __name__ == "__main__":
    unittest.main()
