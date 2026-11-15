# SPDX-License-Identifier: GPL-3.0-or-later
"""Tabla de símbolos de una página: lo que vale cada nombre definido en los renglones.

Las expresiones se analizan SIEMPRE sin sustituir (`x` es el símbolo `x`); después `Vista.resolver` aplica lo que vale
en un renglón, en este orden: llamadas a funciones y alias (identidades, también dentro de derivadas e integrales),
protección de las variables ligadas (integración, derivada, índice, límite) y, por último, los valores. Así `∫2x dx`
con `x = 5` no se convierte en `∫10 d5`, y `dy/dx` con `y = x²` da `2x`.

Tipos de definición: `valor` (x = 5), `cantidad` (m = 70 kg), `expr` (y = x², alias simbólico) y `funcion`
(f(x) = x² + 1, un parámetro).
"""

import re
from dataclasses import dataclass, field

import sympy as sp
from sympy.core.function import AppliedUndef

from . import glossary as gl
from .evaluate import MUL_OPS, RELATIONS, ParseError, _Parser, _split, accent_token
from .layout import to_latex

_LIGA = (sp.Integral, sp.Sum, sp.Product, sp.Derivative, sp.Limit)


@dataclass
class Definicion:
    renglon: int
    nombre: str  # "x", "x_1", "yhat", "f"
    tipo: str  # "valor" | "cantidad" | "expr" | "funcion"
    param: str | None = None  # solo "funcion" (un parámetro)
    expr: object = None  # sympy, sin sustituir
    cantidad: object = None  # pint
    _texto: str | None = field(default=None, compare=False, repr=False)

    def texto(self):
        """Lo que define, como texto (para la clave de la caché; sin el renglón)."""
        if self._texto is None:
            self._texto = f"{self.nombre}|{self.tipo}|{self.param}|{self.expr}|{self.cantidad}"
        return self._texto


class Entorno:
    def __init__(self, definiciones):
        self.definiciones = definiciones

    def vista(self, i, solo_encima=False):
        """Lo que vale en el renglón `i`: la última definición de cada nombre por encima; si no hay ninguna encima,
        la primera de la página (salvo con `solo_encima`)."""
        vale = {}
        for d in self.definiciones:
            if d.renglon < i:
                vale[d.nombre] = d
        if not solo_encima:
            for d in self.definiciones:
                vale.setdefault(d.nombre, d)
        return Vista(vale)


class Vista:
    def __init__(self, vale):
        ds = vale.values()
        self._clave = "|".join(sorted(d.texto() for d in ds))
        self.funciones = {d.nombre: (sp.Symbol(d.param), d.expr) for d in ds if d.tipo == "funcion"}
        self.alias = {d.nombre: d.expr for d in ds if d.tipo == "expr"}
        self.valores = {d.nombre: d.expr for d in ds if d.tipo == "valor"}
        self.cantidades = {d.nombre: d.cantidad for d in ds if d.tipo == "cantidad"}

    @property
    def conocidas(self):
        return set(self.alias) | set(self.valores)

    def clave(self):
        """Texto de todo lo que vale, para la caché por renglón."""
        return self._clave

    def resolver(self, expr):
        """(expr con lo que vale aquí, ¿cambió algo?). Un ciclo de alias lanza ParseError."""
        if not (self.funciones or self.alias or self.valores):
            return expr, False
        original = expr
        for _ in range(8):
            ligadas = _ligadas(expr)
            nuevo = expr.replace(self._es_llamada, self._llamada)
            sustituir = {sp.Symbol(n): e for n, e in self.alias.items() if n not in ligadas}
            nuevo = nuevo.xreplace(sustituir) if sustituir else nuevo
            if nuevo == expr:
                break
            expr = nuevo
        if any(n in self.alias and n not in _ligadas(expr) for n in (s.name for s in expr.free_symbols)):
            raise ParseError("definiciones que se llaman unas a otras")
        mapa = {}
        expr = expr.replace(lambda n: isinstance(n, _LIGA), lambda n: _proteger(n, mapa))
        if self.valores:
            expr = expr.xreplace({sp.Symbol(n): v for n, v in self.valores.items()})
        expr = expr.xreplace(mapa)
        return expr, expr != original

    def _es_llamada(self, e):
        return isinstance(e, AppliedUndef) and len(e.args) == 1 and _nombre_llamada(e)[0] in self.funciones

    def _llamada(self, e):
        nombre, n = _nombre_llamada(e)
        param, cuerpo = self.funciones[nombre]
        return (sp.diff(cuerpo, param, n) if n else cuerpo).xreplace({param: e.args[0]})


def _nombre_llamada(e):
    """(nombre, orden de derivada) de una llamada: f(a) -> ("f", 0); f'(a) = `f__d1(a)` -> ("f", 1)."""
    m = re.fullmatch(r"(.+)__d(\d+)", e.func.__name__)
    return (m.group(1), int(m.group(2))) if m else (e.func.__name__, 0)


def _ligadas(expr):
    """Nombres de las variables ligadas de `expr`: de integración, de derivada, índices de ∑ y ∏ y de límite."""
    out = set()
    for n in sp.preorder_traversal(expr):
        if isinstance(n, (sp.Integral, sp.Sum, sp.Product)):
            out |= {l[0] for l in n.limits}
        elif isinstance(n, sp.Derivative):
            out |= set(n.variables)
        elif isinstance(n, sp.Limit):
            out.add(n.args[1])
    return {s.name for s in out if isinstance(s, sp.Symbol)}


def _proteger(n, mapa):
    """`n` con sus variables ligadas cambiadas por Dummy (el mapa inverso se acumula en `mapa`)."""

    def nuevas(vs):
        ds = {v: sp.Dummy(v.name) for v in vs if isinstance(v, sp.Symbol)}
        mapa.update({d: v for v, d in ds.items()})
        return ds

    if isinstance(n, sp.Derivative):
        ds = nuevas(n.variables)
        return sp.Derivative(n.expr.xreplace(ds), *[(ds.get(v, v), c) for v, c in n.variable_count])
    if isinstance(n, sp.Limit):
        e, z, z0, sentido = n.args
        ds = nuevas([z])
        return sp.Limit(e.xreplace(ds), ds.get(z, z), z0, sentido)
    ds = nuevas([l[0] for l in n.limits])
    return type(n)(n.function.xreplace(ds), *[(ds.get(l[0], l[0]), *l[1:]) for l in n.limits])


def _value(expr):
    v = expr.doit()
    return v if v.is_Rational else sp.simplify(v)


def _unit_text(tokens):
    """Texto de unas fichas que podrían ser una unidad (kg, m/s^2, µm)."""
    out = ""
    for t in tokens:
        if t[0] != "sym" or t[3] is not None:
            raise ParseError("no es una unidad")
        v = t[1]
        if v == r"\mu":
            v = "µ"
        elif v == r"\Omega":
            v = "Ω"
        elif v in MUL_OPS:
            v = "*"
        elif v not in ("/",) and not v.isalpha():
            raise ParseError("no es una unidad")
        out += v
        if t[2] is not None:
            out += "**" + to_latex(t[2])
    return out


def definir(tokens, i, vista, units=None):
    """La definición que hace el renglón `i` (fichas `tokens`) o None. `vista`: lo que vale encima. `units`: sin uso
    (las constantes con unidad del glosario se resuelven al calcular)."""
    partes = _split(tokens, RELATIONS)
    if len(partes) != 2 or partes[0][1] != "=" or not partes[1][0]:
        return None
    lhs, rhs = partes[0][0], partes[1][0]
    if len(lhs) == 1 and lhs[0][0] == "accent":
        try:
            lhs = [accent_token(lhs[0])]
        except ParseError:
            return None
    nombre = _nombre(lhs[:1])
    param = None
    if len(lhs) == 4 and _es(lhs[1], "(") and _es(lhs[3], ")"):
        param = _nombre(lhs[2:3])
        if param is None or param == nombre or nombre in vista.valores:
            return None
    elif len(lhs) != 1:
        return None
    if nombre is None:
        return None
    # número con unidad: lo que sigue a la primera ficha numérica es la unidad
    if param is None and len(rhs) >= 2 and rhs[0][0] == "num" and rhs[0][2] is None:
        try:
            q = gl.registry().Quantity(float(rhs[0][1]), _unit_text(rhs[1:]))
            if not q.dimensionless:
                return Definicion(i, nombre, "cantidad", cantidad=q)
        except Exception:  # noqa: BLE001 - no era una unidad
            pass
    try:
        expr = _Parser(rhs, {}, vista.funciones, vista.conocidas).full()
    except Exception:  # noqa: BLE001 - no se entiende: no es una definición
        return None
    libres = {s.name for s in expr.free_symbols} - {param}
    if nombre in libres:
        return None
    if param is not None:
        return Definicion(i, nombre, "funcion", param=param, expr=expr)
    if not libres:
        try:
            valor = _value(expr)
        except Exception:  # noqa: BLE001
            return None
        return Definicion(i, nombre, "valor", expr=valor) if valor.is_number and not valor.free_symbols else None
    return Definicion(i, nombre, "expr", expr=expr)


def _es(tok, v):
    return tok[0] == "sym" and tok[1] == v and tok[2] is None and tok[3] is None


def _nombre(fichas):
    """Nombre de una variable de una sola ficha (x, x_1, \\theta), o None."""
    if len(fichas) != 1 or fichas[0][0] != "sym" or fichas[0][2] is not None:
        return None
    try:
        s = _Parser([], {"e": sp.Symbol("e")})._symbol(fichas[0])
    except ParseError:
        return None
    return s.name if isinstance(s, sp.Symbol) else None
