# SPDX-License-Identifier: GPL-3.0-or-later
"""Diccionario de símbolos y el campo «Otro…» (PLAN-PENDIENTE.md §2, punto 5)."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from onepepe_math import nombres  # noqa: E402


class TestNombres(unittest.TestCase):
    def test_name_char_latex_and_shortcut_give_the_same(self):
        for text in ["delta", "Delta ", "δ", r"\delta", r"\d"]:
            self.assertEqual(nombres.resolve(text), r"\delta" if text != "Delta " else r"\delta")
        self.assertEqual(nombres.resolve(r"\D"), r"\Delta")
        self.assertEqual(nombres.resolve("alfa"), r"\alpha")
        self.assertEqual(nombres.resolve("alpha"), r"\alpha")
        self.assertEqual(nombres.resolve("menor o igual"), r"\leq")
        self.assertEqual(nombres.resolve("<="), r"\leq")
        self.assertEqual(nombres.resolve("!="), r"\neq")
        self.assertEqual(nombres.resolve("->"), r"\rightarrow")
        self.assertEqual(nombres.resolve("=>"), r"\Rightarrow")
        self.assertEqual(nombres.resolve("inf"), r"\infty")
        self.assertEqual(nombres.resolve("raiz"), r"\sqrt")
        self.assertEqual(nombres.resolve("x"), "x")
        self.assertEqual(nombres.resolve(r"\longrightarrow"), r"\rightarrow")
        self.assertIsNone(nombres.resolve("nada de nada"))

    def test_search_suggests_while_typing(self):
        found = [e["latex"] for e in nombres.search("sum")]
        self.assertEqual(found[0], r"\sum")
        found = [e["latex"] for e in nombres.search("men")]
        self.assertIn(r"\leq", found)
        self.assertIn("-", found)
        self.assertEqual(nombres.search(""), [])

    def test_every_entry_is_complete(self):
        seen = set()
        for e in nombres.entries():
            self.assertIn(e["group"], nombres.GROUPS)
            self.assertTrue(e["names"] and e["char"])
            self.assertNotIn(e["latex"], seen)
            seen.add(e["latex"])
        # casi todos se pueden dibujar limpios
        with_glyph = sum(e["glyph"] for e in nombres.entries())
        self.assertGreater(with_glyph, 0.8 * len(seen))

    def test_server_command(self):
        from onepepe_math.server import Server

        server = Server(classifier=object(), personal=[])
        r = server.handle({"cmd": "names", "query": r"\d"})
        self.assertEqual(r["exact"], r"\delta")
        self.assertEqual(r["entries"][0]["latex"], r"\delta")
        r = server.handle({"cmd": "names"})
        self.assertIn("griego", r["groups"])


if __name__ == "__main__":
    unittest.main()
