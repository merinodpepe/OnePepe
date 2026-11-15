# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests de la tabla de símbolos de la página (`entorno.py`)."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import sympy as sp  # noqa: E402

from onepepe_math.entorno import Definicion, Entorno, Vista, definir  # noqa: E402
from onepepe_math.evaluate import ParseError, _tokens  # noqa: E402


def toks(*items):
    return _tokens(("row", [("sym", it) if isinstance(it, str) else it for it in items]))


x, y, t, f = sp.symbols("x y t f")


class TestVista(unittest.TestCase):
    def test_ultima_encima_y_primera_de_la_pagina(self):
        ent = Entorno([Definicion(1, "x", "valor", expr=sp.Integer(5)), Definicion(3, "x", "valor", expr=sp.Integer(7))])
        self.assertEqual(ent.vista(2).valores, {"x": 5})
        self.assertEqual(ent.vista(4).valores, {"x": 7})
        # nada encima: la primera de la página, salvo con solo_encima
        self.assertEqual(ent.vista(0).valores, {"x": 5})
        self.assertEqual(ent.vista(0, solo_encima=True).valores, {})
        # una definición en el propio renglón no cuenta como «encima»
        self.assertEqual(ent.vista(1, solo_encima=True).valores, {})


class TestDefinir(unittest.TestCase):
    def d(self, *items, vista=None):
        return definir(toks(*items), 0, vista or Vista({}), None)

    def test_tipos(self):
        d = self.d("x", "=", "5")
        self.assertEqual((d.nombre, d.tipo, d.expr), ("x", "valor", 5))
        self.assertEqual(self.d("y", "=", "2", "x", "+", "3").tipo, "expr")
        d = self.d("f", "(", "x", ")", "=", "x", ("sup", ("sym", "x"), ("row", [("sym", "2")])), "+", "1")
        self.assertEqual((d.nombre, d.tipo, d.param), ("f", "funcion", "x"))

    def test_cantidad(self):
        d = self.d("m", "=", "7", "0", "k", "g")
        self.assertEqual((d.nombre, d.tipo), ("m", "cantidad"))
        self.assertEqual(d.cantidad.to("kg").magnitude, 70)

    def test_no_son_definiciones(self):
        self.assertIsNone(self.d("x", "=", "x", "+", "1"))  # ecuación
        self.assertIsNone(self.d("2", "x", "=", "6"))
        self.assertIsNone(self.d("x", "=", "5", "="))  # acaba en «=»: es una pregunta
        self.assertIsNone(self.d("f", "(", "f", ")", "=", "1"))
        self.assertIsNone(self.d("f", "(", "x", ")", "=", "x", vista=Vista({"f": Definicion(0, "f", "valor", expr=sp.Integer(1))})))


class TestResolver(unittest.TestCase):
    def vista(self, *defs):
        return Entorno(list(defs)).vista(99)

    def test_alias_antes_de_proteger_valores_despues(self):
        v = self.vista(Definicion(0, "y", "expr", expr=x**2), Definicion(1, "x", "valor", expr=sp.Integer(5)))
        # dy/dx con y = x²: el alias entra en la derivada y x (variable ligada) no se sustituye
        r, cambiado = v.resolver(sp.Derivative(y, x))
        self.assertTrue(cambiado)
        self.assertEqual(r.doit(), 2 * x)
        # una x libre sí se sustituye
        self.assertEqual(v.resolver(x + 1)[0], 6)
        # ∫2x dx no es ∫10 d5
        self.assertEqual(v.resolver(sp.Integral(2 * x, x))[0].doit(), x**2)

    def test_llamadas_y_primas(self):
        v = self.vista(Definicion(0, "f", "funcion", param="x", expr=x**2 + 1))
        F = sp.Function
        self.assertEqual(v.resolver(F("f")(3))[0], 10)
        self.assertEqual(v.resolver(F("f__d1")(x))[0], 2 * x)
        self.assertEqual(v.resolver(F("f")(x))[0], x**2 + 1)
        self.assertEqual(v.resolver(F("g")(x)), (F("g")(x), False))

    def test_ciclo_no_se_cuelga(self):
        v = self.vista(Definicion(0, "x", "expr", expr=y + 1), Definicion(1, "y", "expr", expr=x + 1))
        with self.assertRaises(ParseError):
            v.resolver(x)

    def test_sin_cambios(self):
        self.assertEqual(Vista({}).resolver(x + 1), (x + 1, False))


if __name__ == "__main__":
    unittest.main()
