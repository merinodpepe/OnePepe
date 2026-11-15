# SPDX-License-Identifier: GPL-3.0-or-later
"""Léxico: números de varias cifras, cifras mal leídas dentro de un número y derivada segunda."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from sondas_util import H, compone  # noqa: E402

from onepepe_math.layout import Sym, parse  # noqa: E402
from onepepe_math.lexico import corregir_entidades  # noqa: E402


def fila(*cosas, gap=2.0):
    """Símbolos en una línea: etiqueta o (etiqueta, alto relativo); base común, 8 de ancho."""
    out, x = [], 0.0
    for c in cosas:
        lab, sc = c if isinstance(c, tuple) else (c, 1.0)
        w = 3 if lab in (",", ".") else 8
        h = H * sc if lab not in (",", ".") else 2.4
        out.append(Sym(lab, (x, 24.0 - h, x + w, 24.0)))
        x += w + gap
    return out


def nodos(syms):
    return parse(syms)[1]


class TestNumeros(unittest.TestCase):
    def test_numero_con_cifra_pequena(self):
        self.assertEqual(nodos(fila("3", ("0", 0.75))), [("num", "30")])

    def test_hueco_separa(self):
        r = nodos(hueco_numeros_syms())
        self.assertEqual(r[:3], [("num", "12"), ("sym", r"\,"), ("num", "34")])

    def test_decimal(self):
        self.assertEqual(nodos(fila("3", ",", "5")), [("num", "3.5")])

    def test_punto_multiplica(self):
        self.assertEqual(nodos(fila("3", ".", "4", "=")), [("num", "3"), ("sym", "."), ("num", "4"), ("sym", "=")])
        self.assertEqual([m["text"] for m in compone("3.4=")[-1]["marks"]], ["12"])

    def test_exponente_no_es_numero(self):
        self.assertEqual(nodos(_potencia())[0][:2], ("sup", ("num", "2")))
        self.assertEqual(nodos(_potencia())[0][2], ("row", [("num", "10")]))


def hueco_numeros_syms():
    from sondas_util import hueco_numeros as hn

    return [Sym(s["latex"], tuple(s["box"])) for s in hn()["symbols"]]


def _potencia():
    out = fila("2")
    out += [Sym("1", (9, 10, 13, 16)), Sym("0", (14, 10, 18, 16))]
    return out


def cands(*pares):
    return [list(p) for p in pares]


class TestEntidades(unittest.TestCase):
    CAJAS = [(0, 4, 8, 24), (10, 4, 18, 24), (20, 4, 28, 24)]

    def test_corrige_S_entre_cifras(self):
        c = [[("1", 0.9)], [("S", 0.6), ("5", 0.2)], [("3", 0.9)]]
        self.assertEqual(corregir_entidades(c, self.CAJAS, H)[1][0][0], "5")

    def test_no_corrige_2b(self):
        c = [[("2", 0.9)], [("b", 0.8), ("6", 0.05)]]
        self.assertEqual(corregir_entidades(c, self.CAJAS[:2], H)[1][0][0], "b")

    def test_no_toca_fijos(self):
        c = [[("1", 0.9)], [("S", 1.0)], [("3", 0.9)]]
        self.assertEqual(corregir_entidades(c, self.CAJAS, H, fijos={1})[1][0][0], "S")


class TestDerivadaSegunda(unittest.TestCase):
    def test_segunda_derivada_y(self):
        r = compone("y=x^3", r"\frac{d^2y}{dx^2}=", variables=True)[-1]
        self.assertEqual([m["text"] for m in r["marks"]], ["6*x"])


if __name__ == "__main__":
    unittest.main()
