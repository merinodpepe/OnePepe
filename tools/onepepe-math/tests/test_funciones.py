# SPDX-License-Identifier: GPL-3.0-or-later
"""Funciones escritas a mano (sin, ln, log₂, sen...): cada fila de la tabla de PLAN-PENDIENTE.md §2, punto 7."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from onepepe_math.evaluate import answer  # noqa: E402
from onepepe_math.layout import Sym, parse, to_latex  # noqa: E402

# alturas de una letra a mano (y hacia abajo): las de x-altura, las altas y las que bajan
SHORT = set("acemnorsuvwxz+=-")
TALL = set("bdfhiklt0123456789()|")


def write(text, alts=None, x=0.0, scale=1.0):
    """Símbolos de una fila escrita a mano. `text`: lista de etiquetas (o una cadena)."""
    out = []
    alts = alts or {}
    for i, lab in enumerate(text):
        if lab in TALL or lab.isdigit():
            y0, y1 = 4, 20
        elif lab == "=":
            y0, y1 = 12, 17
        elif lab in ("-", "+"):
            y0, y1 = 11, 18
        elif lab in "gjpqy":
            y0, y1 = 10, 26
        else:
            y0, y1 = 10, 20
        out.append(Sym(lab, (x, y0 * scale, x + 8 * scale, y1 * scale), tuple(alts.get(i, ()))))
        x += 10 * scale
    return out


def result(syms):
    r = answer(parse(syms))
    return None if r is None else r["text"]


class TestFunctions(unittest.TestCase):
    def test_with_parentheses(self):
        self.assertEqual(result(write(list("ln(1)="))), "0")
        self.assertEqual(result(write(list("log(100)="))), "2")
        self.assertEqual(result(write(list("sin(0)="))), "0")
        self.assertEqual(result(write(list("exp(0)="))), "1")

    def test_ln_is_shown_as_ln(self):
        r = result(write(list("ln2=")))
        self.assertTrue(r.startswith("ln(2) ≈ 0.69"), r)

    def test_argument_without_parentheses_is_not_a_subscript(self):
        # la «x» es más baja que «sin» (la «i» sube): antes salía \sin_{x}
        syms = write(list("sinx"))
        self.assertEqual(to_latex(parse(syms)), r"\sin x")
        self.assertEqual(result(write(list("sin0="))), "0")
        self.assertEqual(result(write(list("ln1="))), "0")

    def test_log_base(self):
        syms = write(list("log")) + [Sym("2", (31, 17, 36, 25))] + write(list("(8)="), x=38)
        self.assertEqual(to_latex(parse(syms))[:7], r"\log_{2")
        self.assertEqual(result(syms), "3")

    def test_l_read_as_one(self):
        # «ln(1)» con la «l» leída como «1», pero «l» estaba entre sus candidatas
        syms = write(list("1n(1)="), alts={0: ("1", "l")})
        self.assertEqual(result(syms), "0")
        # sin «l» entre las candidatas, es 1·n·1
        syms = write(list("1n(1)="), alts={0: ("1", "7")})
        self.assertNotEqual(result(syms), "0")
        # «c0s(0)»
        self.assertEqual(result(write(list("c0s(0)="), alts={1: ("0", "o")})), "1")
        # «10» sigue siendo diez
        self.assertEqual(result(write(list("10+1="))), "11")

    def test_spanish_names(self):
        self.assertEqual(to_latex(parse(write(list("senx")))), r"\sin x")
        self.assertEqual(result(write(list("sen(0)="))), "0")
        self.assertEqual(result(write(list("tg(0)="))), "0")
        self.assertEqual(to_latex(parse(write(list("arctg(1)")))), r"\arctan(1)")

    def test_argument_runs_to_the_next_operator(self):
        # sin 2x = sin(2x); sin x cos x = sin(x)·cos(x)
        self.assertEqual(result(write(list("sin2x-sin(2x)="))), "0")
        self.assertEqual(result(write(list("sinxcosx-sin(x)cos(x)="))), "0")

    def test_x_between_numbers_is_times(self):
        self.assertEqual(result(write(list("3x4="))), "12")

    def test_nth_root(self):
        root = Sym(r"\sqrt", (6, 2, 40, 22))
        index = Sym("3", (4, 3, 9, 10))
        body = write(list("27"), x=16)
        syms = [index, root] + body + [Sym("=", (44, 12, 52, 17))]
        self.assertEqual(to_latex(parse(syms)), r"\sqrt[3]{27}=")
        self.assertEqual(result(syms), "3")


if __name__ == "__main__":
    unittest.main()
