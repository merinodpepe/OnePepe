# SPDX-License-Identifier: GPL-3.0-or-later
"""Renglones de una página: «=» con «?», variables, revisión de pasos, LaTeX y formas de una fracción."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from test_funciones import write  # noqa: E402

from onepepe_math import pagina  # noqa: E402


def row(text, y=0.0, x=0.0):
    """Un renglón escrito a mano, a la altura `y` (cada renglón, 40 unidades más abajo)."""
    syms = write(list(text) if isinstance(text, str) else text, x=x)
    return {"symbols": [{"latex": s.label, "box": [s.box[0], s.box[1] + y, s.box[2], s.box[3] + y]} for s in syms]}


def page(*texts, **kw):
    rows = [row(t, 40.0 * i) for i, t in enumerate(texts)]
    return pagina.page(rows, **kw)


def marks(result, i):
    return [(m["kind"], m["text"]) for m in result[i]["marks"]]


class TestEquals(unittest.TestCase):
    def test_result_and_question_mark(self):
        r = page("2+3=", "2+=")
        self.assertEqual(marks(r, 0), [("value", "5")])
        # no se puede calcular: «?» con lo que se ha leído
        self.assertEqual(marks(r, 1)[0], ("unknown", "?"))
        self.assertEqual(r[1]["marks"][0]["latex"], "2+=")
        # al lado del «=»
        self.assertGreater(r[0]["marks"][0]["anchor"][0], 38)

    def test_fraction_formats(self):
        r = page("7/2=")
        formats = [f["text"] for f in r[0]["marks"][0]["formats"]]
        self.assertEqual(formats, ["7/2", "3.5", "3 1/2"])
        self.assertEqual(pagina.number_formats(4), [])

    def test_off(self):
        self.assertEqual(marks(page("2+3=", equals=False), 0), [])


class TestVariables(unittest.TestCase):
    def test_define_and_use(self):
        r = page("x=3", "2x+1", "x+y=", "y=2x", variables=True)
        self.assertEqual(marks(r, 0), [])
        self.assertEqual(marks(r, 1), [("variable", "= 7")])
        # y se define más abajo: vale la primera definición de la página
        self.assertEqual(marks(r, 2), [("value", "9")])

    def test_redefining_changes_the_lines_below(self):
        r = page("a=2", "a+1", "a=5", "a+1", variables=True)
        self.assertEqual(marks(r, 1), [("variable", "= 3")])
        self.assertEqual(marks(r, 3), [("variable", "= 6")])

    def test_units(self):
        r = page("m=70kg", "g=10", "mg=", variables=True)
        self.assertEqual(r[2]["marks"][0]["kind"], "value")
        self.assertIn("700", r[2]["marks"][0]["text"])
        self.assertIn("kg", r[2]["marks"][0]["text"])

    def test_off_by_default(self):
        self.assertEqual(marks(page("x=3", "2x+1"), 1), [])


class TestSteps(unittest.TestCase):
    def check(self, *texts):
        r = page(*texts, check=True, equals=False)
        return [m for i in range(len(texts)) for m in marks(r, i)]

    def test_good_steps_are_not_marked(self):
        self.assertEqual(self.check("2x+4=10", "2x=6", "x=3"), [])
        self.assertEqual(self.check("(x+1)2", "=2x+2"), [])
        self.assertEqual(self.check("3(x+1)", "=3x+3"), [])

    def test_term_moved_without_changing_sign(self):
        r = self.check("2x+4=10", "2x=10+4")
        self.assertEqual(len(r), 1)
        self.assertIn("sin cambiarle el signo", r[0][1])

    def test_not_distributing(self):
        r = self.check("3(x+1)", "=3x+1")
        self.assertIn("todos los términos del paréntesis", r[0][1])

    def test_dividing_wrong(self):
        r = self.check("2x=6", "x=4")
        self.assertIn("dividiendo, no restando", r[0][1])

    def test_lost_sign(self):
        r = self.check("5-x", "=5+x")
        self.assertIn("signo", r[0][1])

    def test_unknown_error(self):
        r = self.check("2x=6", "x=7")
        self.assertIn("mismas soluciones", r[0][1])

    def test_steps_in_one_row(self):
        r = page(["2", "x", "=", "6", r"\rightarrow", "x", "=", "4"], check=True)
        self.assertEqual(marks(r, 0)[0][0], "error")
        r = page(["2", "x", "=", "6", r"\rightarrow", "x", "=", "3"], check=True)
        self.assertEqual(marks(r, 0), [])

    def test_square_of_a_sum(self):
        def with_squares(text, y):
            # las «2» van de exponente: pequeñas y arriba
            out = row(text, y)
            for s in out["symbols"]:
                if s["latex"] == "2":
                    s["box"] = [s["box"][0], y + 2, s["box"][0] + 5, y + 9]
            return out

        rows = [with_squares("(a+b)2", 0), with_squares("=a2+b2", 40)]
        r = pagina.page(rows, check=True)
        self.assertIn("2ab", r[1]["marks"][0]["text"])
        rows = [with_squares("(a+b)2", 0), with_squares("=a2+2ab+b2", 40)]
        self.assertEqual(pagina.page(rows, check=True)[1]["marks"], [])


class TestLatex(unittest.TestCase):
    def test_rows(self):
        self.assertEqual(pagina.latex_rows([row("x+1"), row("sin(x)")]), ["x+1", r"\sin(x)"])


class TestServer(unittest.TestCase):
    def test_page_latex_and_plot_commands(self):
        from onepepe_math.server import Server

        server = Server(classifier=object(), personal=[])
        r = server.handle({"cmd": "page", "rows": [row("2+3=")], "equals": True})
        self.assertEqual(r["rows"][0]["marks"][0]["text"], "5")
        r = server.handle({"cmd": "latex", "rows": [row("2+3")]})
        self.assertEqual(r["latex"], ["2+3"])


if __name__ == "__main__":
    unittest.main()
