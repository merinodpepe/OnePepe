# SPDX-License-Identifier: GPL-3.0-or-later
"""Sondas compuestas con LaTeX: valores «Tras T3» (los que hoy fallan, con expectedFailure) y casos de «Qué funciona»
de la auditoría, con la salida actual como valor esperado."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from sondas_util import H, compone, hueco_numeros, num2, sumrow  # noqa: E402


def textos(*filas, **kw):
    """Textos de las marcas del último renglón."""
    return [m["text"] for m in compone(*filas, **kw)[-1]["marks"]]


class TestSondasTrasT3(unittest.TestCase):
    def test_paren_mas(self):
        self.assertEqual(textos("(x+1)+3="), ["x + 4"])

    def test_segunda_derivada(self):
        self.assertEqual(textos(r"\frac{d^2}{dx^2}(x^3)="), ["6*x"])

    def test_dxdt_frac(self):
        self.assertEqual(textos("x=t^2", r"\frac{dx}{dt}=", variables=True), ["?"])

    def test_dydx_frac(self):
        self.assertEqual(textos("y=x^2", r"\frac{dy}{dx}=", variables=True), ["?"])

    def test_dxdt_linea(self):
        self.assertEqual(textos("x=t^2", "dx/dt=", variables=True), ["?"])

    def test_f_de_3(self):
        self.assertEqual(textos("f(x)=x^2+1", "f(3)=", variables=True), ["?"])

    def test_f_de_x(self):
        self.assertEqual(textos("x=5", "f(x)=x^2+1", "f(x)=", variables=True), ["?"])

    def test_unidades(self):
        self.assertRegex(" ".join(textos("m=70kg", "g=9,8m/s^2", "mg=", variables=True)), r"^686.*N$")

    def test_e_a_la_5(self):
        self.assertRegex(" ".join(textos("e^5=")), "148,4")

    def test_exponente_negativo(self):
        # hoy no muestra nada y debe seguir así
        r = compone("e^{-x}")[-1]
        self.assertEqual(r["latex"], "e^{-x}")
        self.assertEqual(r["marks"], [])

    def test_prima(self):
        self.assertEqual(textos("f'(x)="), ["?"])

    def test_prima_latex(self):
        self.assertEqual(compone("f'(x)=")[-1]["latex"], "f'(x)=")

    def test_sumatorio(self):
        self.assertEqual(textos(sumrow()), ["6"])

    def test_cifra_pequena(self):
        self.assertEqual(compone(num2("3", "0", 0.72))[-1]["latex"], "x+30=")

    def test_cifra_exponente(self):
        self.assertEqual(compone(num2("3", "0", 0.6, -0.4 * H))[-1]["latex"], "x+3^{0}=")

    def test_hueco_numeros(self):
        self.assertEqual(textos(hueco_numeros()), ["?"])


class TestQueFunciona(unittest.TestCase):
    """Valores = salida actual de `compone`, que coincide con «Qué funciona» de la auditoría."""

    def test_x5_mas_con_variables(self):
        self.assertEqual(textos("x=5", "2x+3=", variables=True), ["13"])

    def test_cadena_de_definiciones(self):
        self.assertEqual(textos("a=2", "b=3", "c=a+b", "c^2=", variables=True), ["25"])

    def test_redefinir_x_con_x12(self):
        self.assertEqual(textos("x=12", "x+30=", variables=True), ["42"])

    def test_sumas_y_divisiones(self):
        self.assertEqual(textos("12+34="), ["46"])
        self.assertEqual(textos("3,5+1,5="), ["5"])
        self.assertEqual(textos("1000/8="), ["125"])

    def test_dos_por_tres(self):
        self.assertEqual(textos(r"2\times 3="), ["6"])

    def test_derivada_d_dx(self):
        self.assertEqual(textos(r"\frac{d}{dx}(x^2)="), ["2*x"])

    def test_derivada_parcial(self):
        self.assertEqual(textos(r"\frac{\partial}{\partial x}(x^2y)="), ["2*x*y"])


if __name__ == "__main__":
    unittest.main()
