# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests de la estructura y del resultado del "=" (necesitan sympy y pint)."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from onepepe_math.evaluate import answer  # noqa: E402
from onepepe_math.layout import Sym, parse, to_latex  # noqa: E402


def row(*items):
    out = []
    for it in items:
        out.append(("sym", it) if isinstance(it, str) else it)
    return ("row", out)


def sup(base, *exp):
    return ("sup", ("sym", base) if isinstance(base, str) else base, row(*exp))


def sub(base, *s):
    return ("sub", ("sym", base), row(*s))


def frac(num, den):
    return ("frac", num, den)


class TestAnswer(unittest.TestCase):
    def text(self, tree, **kw):
        r = answer(tree, **kw)
        return None if r is None else r["text"]

    def test_arithmetic(self):
        self.assertEqual(self.text(row("3", "+", "4", r"\cdot", "5", "=")), "23")
        self.assertEqual(self.text(row(frac(row("7"), row("8")), "-", frac(row("1"), row("4")), "=")), "5/8")
        self.assertEqual(self.text(row(sup("2", "1", "0"), "=")), "1024")
        self.assertEqual(self.text(row(("sqrt", row("1", "4", "4")), "+", sup("3", "2"), "=")), "21")

    def test_decimals_and_powers_of_ten(self):
        r = answer(row("1", ",", "5", r"\times", "1", sup("0", "-", "2"), "="))
        self.assertEqual(r["text"], "3/200")

    def test_solve(self):
        # 2x+1=7, x=
        self.assertEqual(self.text(row("2", "x", "+", "1", "=", "7", ",", "x", "=")), "x = 3")
        # x^2-5x+6=0 =  (una sola incógnita)
        r = self.text(row(sup("x", "2"), "-", "5", "x", "+", "6", "=", "0", "="))
        self.assertEqual(r, "x = 2, x = 3")

    def test_simplify(self):
        self.assertEqual(self.text(row("2", "a", "+", "3", "a", "-", "b", "=")), "5*a - b")
        # ya simplificado: nada
        self.assertIsNone(self.text(row("(", "a", "+", "b", sup(")", "2"), "=")))

    def test_calculus(self):
        # d/dx (x^3 + 2x) =
        d = frac(row("d"), row("d", "x"))
        self.assertEqual(self.text(row(d, "(", sup("x", "3"), "+", "2", "x", ")", "=")), "3*x**2 + 2")
        # ∫_0^1 x^2 dx =
        integral = ("subsup", ("sym", r"\int"), row("0"), row("1"))
        self.assertEqual(self.text(row(integral, sup("x", "2"), "d", "x", "=")), "1/3")
        # Σ_{i=1}^{4} i =
        s = ("subsup", ("sym", r"\sum"), row("i", "=", "1"), row("4"))
        self.assertEqual(self.text(row(s, "i", "=")), "10")
        # lim_{x→0} sin x / x =
        lim = ("sub", ("sym", r"\lim"), row("x", r"\rightarrow", "0"))
        self.assertEqual(self.text(row(lim, frac(row(r"\sin", "x"), row("x")), "=")), "1")

    def test_greek_and_subscripts(self):
        self.assertEqual(self.text(row(sub("x", "1"), "+", sub("x", "1"), "=")), "2*x_1")
        self.assertEqual(self.text(row("2", r"\pi", "-", r"\pi", "=")), "pi ≈ 3.142")

    def test_units_and_dimension_check(self):
        import pint

        u = pint.UnitRegistry()
        units = {"k_B": 1.380649e-23 * u.J / u.K, "T": 310 * u.K, "m": 2 * u.m, "t": 3 * u.s}
        r = answer(row(sub("k", "B"), "T", "="), units=units)
        self.assertTrue(r["text"].startswith("4.28"), r)
        self.assertIn("zJ", r["text"])  # 4,28·10⁻²¹ J = 4,28 zJ
        # metros + segundos: no hay resultado
        self.assertIsNone(answer(row("m", "+", "t", "="), units=units))

    def test_invalid_gives_nothing(self):
        self.assertIsNone(answer(row("3", "+", "=")))
        self.assertIsNone(answer(row("3", "+", "4")))  # sin "="
        self.assertIsNone(answer(row(")", "3", "=")))


class TestLayout(unittest.TestCase):
    def test_power_subscript_fraction(self):
        # x² + k_B  y  a/b dibujados con cajas como las de una letra a mano (y hacia abajo)
        syms = [
            Sym("x", (0, 10, 10, 20)),
            Sym("2", (11, 2, 16, 9)),
            Sym("+", (20, 11, 28, 19)),
            Sym("k", (32, 4, 42, 20)),
            Sym("B", (43, 16, 48, 24)),
        ]
        self.assertEqual(to_latex(parse(syms)), "x^{2}+k_{B}")
        syms = [Sym("a", (0, 0, 10, 10)), Sym("-", (-2, 14, 14, 15)), Sym("b", (0, 18, 10, 30))]
        self.assertEqual(to_latex(parse(syms)), r"\frac{a}{b}")

    def test_functions(self):
        syms = [Sym(c, (i * 9, 10, i * 9 + 8, 20)) for i, c in enumerate("sin")] + [Sym("x", (30, 10, 38, 20))]
        self.assertEqual(to_latex(parse(syms)), r"\sin x")


if __name__ == "__main__":
    unittest.main()
