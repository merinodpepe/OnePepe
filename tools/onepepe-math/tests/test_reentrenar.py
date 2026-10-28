# SPDX-License-Identifier: GPL-3.0-or-later
"""reentrenar.py --estimar: cuenta tus ejemplos y estima el tiempo (sin entrenar nada)."""

import importlib
import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))


class TestEstimate(unittest.TestCase):
    def test_estimate_counts_symbols_and_uses_last_duration(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "math"))
            with open(os.path.join(d, "math", "correcciones.jsonl"), "w", encoding="utf-8") as f:
                for i in range(5):
                    f.write(json.dumps({"latex": "x", "strokes": [[[0, 0], [1, 1]]], "date": f"2026-10-2{i}"}) + "\n")
                f.write(json.dumps({"latex": "__trozo__", "kind": "junk", "strokes": [[[0, 0], [1, 0]]]}) + "\n")
            with open(os.path.join(d, "math", "ultimo_entreno.json"), "w", encoding="utf-8") as f:
                json.dump({"seconds": 180, "symbols": 2}, f)
            old = os.environ.get("ONEPEPE_DATA")
            os.environ["ONEPEPE_DATA"] = d
            try:
                import reentrenar

                reentrenar = importlib.reload(reentrenar)
                out = io.StringIO()
                with redirect_stdout(out):
                    self.assertEqual(reentrenar.main(["--estimar"]), 0)
                est = json.loads(out.getvalue())
            finally:
                if old is None:
                    del os.environ["ONEPEPE_DATA"]
                else:
                    os.environ["ONEPEPE_DATA"] = old
            self.assertEqual(est["examples"], 5)
            self.assertEqual(est["junk"], 1)
            self.assertEqual(est["new_since_last"], 3)
            self.assertEqual(est["minutes"], 3.0)
            self.assertFalse(est["model"])


if __name__ == "__main__":
    unittest.main()
