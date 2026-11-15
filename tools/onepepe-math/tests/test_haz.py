# SPDX-License-Identifier: GPL-3.0-or-later
"""Las N mejores lecturas y la gramática (M10)."""

import os
import sys
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from onepepe_math import haz  # noqa: E402
from onepepe_math.context import reading_order  # noqa: E402


def fila(*cands):
    """Candidatas por símbolo ([(etiqueta, p)...]) y cajas de 5x10 en una línea, cada 8."""
    return list(cands), [(8.0 * i, 0.0, 8.0 * i + 5, 10.0) for i in range(len(cands))]


def u(x):
    return [(x, 1.0)]


class TestHaz(unittest.TestCase):
    def test_orden_del_arbol(self):
        c = [u("b"), u("-"), u("a")]
        b = [(8.0, 14.0, 12.0, 24.0), (0.0, 11.0, 20.0, 13.0), (9.0, 0.0, 13.0, 10.0)]
        self.assertEqual(reading_order(c, b), [0, 1, 2])  # sin árbol: por el centro
        self.assertEqual(reading_order(c, b, arbol=True), [1, 2, 0])  # raya, a, b

    def test_parentesis(self):
        c, b = fila(u("("), u("x"), u("+"), u("1"), [("l", 0.5), (")", 0.4)])
        self.assertEqual(haz.mejor_lectura(c, b), [0, 0, 0, 0, 1])

    def test_no_cambia_si_ya_vale(self):
        c, b = fila(u("2"), u("+"), u("3"), [("=", 0.55), ("-", 0.4)])
        self.assertEqual(haz.mejor_lectura(c, b), [0, 0, 0, 0])

    def test_margen_grande_no_se_abre(self):
        c, b = fila(u("("), [("2", 0.99), (")", 0.001)], u("+"), u("3"))
        vistos = []
        real = haz.parse
        with mock.patch.object(haz, "parse", lambda s: (vistos.append([x.label for x in s]), real(s))[1]):
            self.assertEqual(haz.mejor_lectura(c, b), [0, 0, 0, 0])
        self.assertTrue(all(")" not in v for v in vistos))

    def test_de_mejor_a_peor(self):
        sal = list(haz._ordenadas([[0, 1, 2], [0, 0.5], [0, 3]]))
        costes = [c for c, _ in sal]
        self.assertEqual(costes, sorted(costes))
        self.assertEqual(len({rs for _, rs in sal}), len(sal))
        self.assertEqual(len(sal), 12)

    def test_fijos_no_se_abren(self):
        c, b = fila(u("("), u("x"), u("+"), u("1"), [("l", 0.5), (")", 0.4)])
        self.assertEqual(haz.mejor_lectura(c, b, fijos={4}), [0, 0, 0, 0, 0])

    def test_histeresis(self):
        c, b = fila(u("("), u("x"), u("+"), u("1"), [("l", 0.9), (")", 0.06)])
        self.assertEqual(haz.mejor_lectura(c, b, premio=1.0), [0] * 5)  # mejora de S: 0,29
        self.assertEqual(haz.mejor_lectura(c, b, premio=2.0), [0, 0, 0, 0, 1])


if __name__ == "__main__":
    unittest.main()
