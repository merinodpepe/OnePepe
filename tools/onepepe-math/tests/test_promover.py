# SPDX-License-Identifier: GPL-3.0-or-later
"""T13: promover_cierre.py con carpetas temporales."""

import hashlib
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from promover_cierre import INSTALAR, promover  # noqa: E402


class Promover(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory()
        r = self.t.name
        self.real, self.trab, self.origen = f"{r}/real", f"{r}/trab", f"{r}/origen.sha256"
        os.makedirs(self.real)
        os.makedirs(self.trab)
        for n in INSTALAR:
            with open(f"{self.trab}/{n}", "wb") as f:
                f.write(b"trabajo-" + n.encode() + b"\n")
        with open(f"{self.trab}/correcciones.jsonl", "ab") as f:
            f.write(b"real-correcciones.jsonl\n")  # la copia de trabajo parte del original
        for n in ("correcciones.jsonl", "simbolos.npz"):
            with open(f"{self.real}/{n}", "wb") as f:
                f.write(b"real-" + n.encode() + b"\n")
        self.escribir_origen()

    def tearDown(self):
        self.t.cleanup()

    def escribir_origen(self):
        with open(self.origen, "w") as f:
            for n in ("correcciones.jsonl", "simbolos.npz"):
                f.write(hashlib.sha256(open(f"{self.real}/{n}", "rb").read()).hexdigest() + f"  {n}\n")

    def leer(self, d, n):
        return open(f"{d}/{n}", "rb").read()

    def test_promover_copia(self):
        self.assertEqual(promover(self.real, self.trab, self.origen, True, sello="S", out=lambda s: None), 0)
        for n in INSTALAR:
            self.assertEqual(self.leer(self.real, n), self.leer(self.trab, n))
        self.assertEqual(self.leer(self.real, "simbolos.npz.S.bak"), b"real-simbolos.npz\n")
        self.assertEqual(self.leer(self.real, "simbolos.anterior.npz"), b"real-simbolos.npz\n")

    def test_promover_informe_no_escribe(self):
        promover(self.real, self.trab, self.origen, False, out=lambda s: None)
        self.assertEqual(self.leer(self.real, "simbolos.npz"), b"real-simbolos.npz\n")
        self.assertFalse(os.path.exists(f"{self.real}/examen.json"))

    def test_promover_detecta_cambios(self):
        with open(f"{self.real}/correcciones.jsonl", "ab") as f:
            f.write(b"nueva\n")
        msgs = []
        promover(self.real, self.trab, self.origen, False, out=msgs.append)
        self.assertTrue(any("CAMBIADO" in m for m in msgs))
        promover(self.real, self.trab, self.origen, True, sello="S", out=lambda s: None)
        self.assertTrue(self.leer(self.real, "correcciones.jsonl").endswith(b"real-correcciones.jsonl\nnueva\n"))

    def test_promover_para_si_cambian_simbolos(self):
        with open(f"{self.real}/simbolos.npz", "ab") as f:
            f.write(b"x")
        self.assertEqual(promover(self.real, self.trab, self.origen, True, out=lambda s: None), 1)
        self.assertFalse(os.path.exists(f"{self.real}/examen.json"))

    def test_sin_confianza(self):
        promover(self.real, self.trab, self.origen, True, sin_confianza=True, out=lambda s: None)
        self.assertFalse(os.path.exists(f"{self.real}/confianza.json"))
        self.assertTrue(os.path.exists(f"{self.real}/examen.json"))


if __name__ == "__main__":
    unittest.main()
