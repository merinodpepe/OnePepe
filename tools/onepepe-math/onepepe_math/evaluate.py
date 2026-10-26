# SPDX-License-Identifier: GPL-3.0-or-later
"""Del árbol de la expresión a SymPy, y el resultado del "=".

No se usa `sympy.parsing.latex` (solo entiende 19 de las 30 expresiones de prueba): el árbol de `layout.py` se recorre
con un analizador propio. Comportamiento (ver MATEMATICAS-INFORME.md, sección 0.3):

- `expr =` sin incógnitas: el valor (exacto si es una fracción sencilla; si no, 4 cifras significativas).
- `ecuación, x =`: despeja x. `ecuación =` con una sola incógnita: la despeja.
- `expr =` con variables: la forma simplificada, solo si es distinta y no más larga.
- derivadas `d/dx(...)`, integrales, sumatorios y límites: se calculan.
- constantes y unidades del glosario: se calculan con pint; si las dimensiones no casan, no hay resultado.
- cualquier cosa ambigua o inválida: None (no se muestra nada).
"""

import sympy as sp

from .latex_tokens import visible_tokens
from .symbols import GREEK

GREEK_NAMES = {g: g[1:] for g in GREEK}
FUNCS = {
    r"\sin": sp.sin, r"\cos": sp.cos, r"\tan": sp.tan, r"\cot": sp.cot, r"\sec": sp.sec, r"\csc": sp.csc,
    r"\sinh": sp.sinh, r"\cosh": sp.cosh, r"\tanh": sp.tanh, r"\arcsin": sp.asin, r"\arccos": sp.acos,
    r"\arctan": sp.atan, r"\ln": sp.log, r"\log": lambda x: sp.log(x, 10), r"\exp": sp.exp,
}
MUL_OPS = {r"\times", r"\cdot", "*"}
DIV_OPS = {"/", r"\div"}
RELATIONS = {"=", r"\neq", "<", ">", r"\leq", r"\geq", r"\approx"}


class ParseError(Exception):
    pass


# ---------- árbol -> tokens ----------


def _tokens(row, decimal_comma=True):
    """Fichas de una fila del árbol: (tipo, valor, exponente, subíndice)."""
    assert row[0] == "row"
    out = []
    for node in row[1]:
        sup = sub = None
        base = node
        if node[0] == "sup":
            base, sup = node[1], node[2]
        elif node[0] == "sub":
            base, sub = node[1], node[2]
        elif node[0] == "subsup":
            base, sub, sup = node[1], node[2], node[3]
        if base[0] == "sym":
            out.append(["sym", base[1], sup, sub])
        else:
            out.append([base[0], base, sup, sub])
    # unir cifras (y el separador decimal) en números
    merged = []
    for t in out:
        prev = merged[-1] if merged else None
        is_digit = t[0] == "sym" and t[1].isdigit()
        if is_digit and prev is not None and prev[0] == "num" and prev[2] is None and prev[3] is None:
            prev[1] += t[1]
            prev[2], prev[3] = t[2], t[3]
            continue
        if (
            t[0] == "sym"
            and t[1] in ({".", ","} if decimal_comma else {"."})
            and prev is not None
            and prev[0] == "num"
            and "." not in prev[1]
            and prev[2] is None
        ):
            prev[1] += "."
            prev.append(t[1])  # separador original, por si no siguen decimales
            continue
        if is_digit:
            merged.append(["num", t[1], t[2], t[3]])
        else:
            merged.append(t)
    # un número que termina en "." sin decimales: el punto era otra cosa
    final = []
    for t in merged:
        if t[0] == "num" and t[1].endswith("."):
            final.append(["num", t[1][:-1], None, None])
            final.append(["sym", t[4] if len(t) > 4 else ".", t[2], t[3]])
        else:
            final.append(t)
    return final


# ---------- analizador ----------


class _Parser:
    def __init__(self, tokens, glossary):
        self.t = tokens
        self.i = 0
        self.glossary = glossary or {}

    def peek(self, k=0):
        j = self.i + k
        return self.t[j] if j < len(self.t) else None

    def take(self):
        tok = self.peek()
        self.i += 1
        return tok

    def is_sym(self, tok, *values):
        return tok is not None and tok[0] == "sym" and tok[1] in values

    def expr(self):
        node = self.term()
        while self.is_sym(self.peek(), "+", "-", r"\pm"):
            op = self.take()[1]
            if op == r"\pm":
                raise ParseError("± no se calcula")
            rhs = self.term()
            node = node + rhs if op == "+" else node - rhs
        return node

    def term(self):
        node = self.unary()
        while True:
            tok = self.peek()
            if tok is None:
                return node
            if self.is_sym(tok, *MUL_OPS):
                self.take()
                node = node * self.unary()
            elif self.is_sym(tok, *DIV_OPS):
                self.take()
                node = node / self.unary()
            elif self._starts_atom(tok):
                node = node * self.unary()
            else:
                return node

    def unary(self):
        if self.is_sym(self.peek(), "-"):
            self.take()
            return -self.unary()
        if self.is_sym(self.peek(), "+"):
            self.take()
            return self.unary()
        return self.atom()

    def _starts_atom(self, tok):
        if tok[0] in ("num", "frac", "sqrt"):
            return True
        if tok[0] != "sym":
            return False
        v = tok[1]
        return (
            len(v) == 1
            and v.isalpha()
            or v in GREEK_NAMES
            or v in FUNCS
            or v in ("(", "[", r"\{", r"\int", r"\sum", r"\prod", r"\lim", r"\infty", r"\partial", r"\sqrt")
        )

    def _apply_scripts(self, value, tok):
        sup = tok[2]
        if sup is not None:
            value = value ** sub_expr(sup, self.glossary)
        return value

    def atom(self):
        tok = self.take()
        if tok is None:
            raise ParseError("falta un término")
        kind = tok[0]
        if kind == "num":
            value = sp.Rational(tok[1]) if "." in tok[1] else sp.Integer(tok[1])
            if tok[3] is not None:
                raise ParseError("subíndice en un número")
            return self._apply_scripts(value, tok)
        if kind == "frac":
            _, num, den = tok[1]
            # derivada d/dx
            num_t, den_t = _tokens(num), _tokens(den)
            if (
                len(num_t) == 1
                and num_t[0][0] == "sym"
                and num_t[0][1] in ("d", r"\partial")
                and len(den_t) == 2
                and den_t[0][0] == "sym"
                and den_t[0][1] in ("d", r"\partial")
                and den_t[1][0] == "sym"
            ):
                var = self._symbol(den_t[1])
                return sp.Derivative(self.unary(), var)
            value = sub_expr(num, self.glossary) / sub_expr(den, self.glossary)
            return self._apply_scripts(value, tok)
        if kind == "sqrt":
            return self._apply_scripts(sp.sqrt(sub_expr(tok[1][1], self.glossary)), tok)
        if kind != "sym":
            raise ParseError(f"no sé leer {kind}")
        v = tok[1]
        if v in ("(", "[", r"\{"):
            close = {"(": ")", "[": "]", r"\{": r"\}"}[v]
            inner = self.expr()
            end = self.take()
            if end is None or end[1] != close:
                raise ParseError("paréntesis sin cerrar")
            return self._apply_scripts(inner, end)
        if v in FUNCS:
            arg = self.unary() if not self.is_sym(self.peek(), "(") else self.atom()
            value = FUNCS[v](arg)
            return self._apply_scripts(value, tok)
        if v == r"\int":
            return self._integral(tok)
        if v in (r"\sum", r"\prod"):
            return self._sum(tok, v)
        if v == r"\lim":
            return self._limit(tok)
        if v == r"\infty":
            return sp.oo
        return self._apply_scripts(self._symbol(tok), tok)

    def _symbol(self, tok):
        v = tok[1]
        name = GREEK_NAMES.get(v, v)
        if not (name.isalpha()):
            raise ParseError(f"símbolo inesperado {v}")
        if tok[3] is not None:
            name = f"{name}_{_plain(tok[3])}"
        if name in self.glossary:
            return self.glossary[name]
        if name == "pi":
            return sp.pi
        return sp.Symbol(name)

    def _integral(self, tok):
        lower = sub_expr(tok[3], self.glossary) if tok[3] is not None else None
        upper = sub_expr(tok[2], self.glossary) if tok[2] is not None else None
        # el integrando llega hasta "d" seguido de una variable
        start = self.i
        j = start
        while j < len(self.t) - 1 and not (self.is_sym(self.t[j], "d") and self.t[j + 1][0] == "sym"):
            j += 1
        if j >= len(self.t) - 1:
            raise ParseError("integral sin diferencial")
        integrand = _Parser(self.t[start:j], self.glossary).full()
        var = self._symbol(self.t[j + 1])
        self.i = j + 2
        if lower is None and upper is None:
            return sp.Integral(integrand, var)
        if lower is None or upper is None:
            raise ParseError("falta un límite de integración")
        return sp.Integral(integrand, (var, lower, upper))

    def _sum(self, tok, kind):
        if tok[3] is None or tok[2] is None:
            raise ParseError("sumatorio sin límites")
        sub_t = _tokens(tok[3])
        if len(sub_t) < 3 or not self.is_sym(sub_t[1], "="):
            raise ParseError("sumatorio sin índice")
        var = self._symbol(sub_t[0])
        lower = _Parser(sub_t[2:], self.glossary).full()
        upper = sub_expr(tok[2], self.glossary)
        body = self.term()
        cls = sp.Sum if kind == r"\sum" else sp.Product
        return cls(body, (var, lower, upper))

    def _limit(self, tok):
        if tok[3] is None:
            raise ParseError("límite sin variable")
        sub_t = _tokens(tok[3])
        if len(sub_t) < 3 or not self.is_sym(sub_t[1], r"\rightarrow"):
            raise ParseError("límite mal escrito")
        var = self._symbol(sub_t[0])
        to = _Parser(sub_t[2:], self.glossary).full()
        return sp.Limit(self.term(), var, to)

    def full(self):
        value = self.expr()
        if self.peek() is not None:
            raise ParseError(f"sobra {self.peek()[1]}")
        return value


def _plain(row):
    """Texto de un subíndice (k_{B} -> "B", x_{12} -> "12")."""
    out = ""
    for node in row[1]:
        if node[0] != "sym":
            raise ParseError("subíndice complejo")
        out += GREEK_NAMES.get(node[1], node[1])
    return out


def sub_expr(row, glossary=None):
    return _Parser(_tokens(row), glossary).full()


def _split(tokens, values):
    parts, cur = [], []
    for t in tokens:
        if t[0] == "sym" and t[1] in values and t[2] is None and t[3] is None:
            parts.append((cur, t[1]))
            cur = []
        else:
            cur.append(t)
    parts.append((cur, None))
    return parts


# ---------- resultado del "=" ----------


def _format_number(value):
    value = sp.nsimplify(value) if isinstance(value, sp.Float) else value
    if isinstance(value, sp.Rational) and (value.q <= 1000 and abs(value.p) < 10**9):
        return value
    return sp.Float(sp.N(value, 4), 4)


def answer(tree, glossary=None, units=None):
    """Resultado del "=" de una fila. Devuelve dict {"latex", "text", "kind"} o None.

    `glossary`: {nombre: valor sympy} (p. ej. constantes); `units`: {nombre: cantidad de pint} para calcular con
    unidades y comprobar dimensiones.
    """
    try:
        return _answer(tree, glossary or {}, units or {})
    except (ParseError, ValueError, TypeError, ZeroDivisionError, NotImplementedError, AttributeError):
        return None
    except Exception as e:  # SymPy puede lanzar errores variados con entradas raras
        if type(e).__module__.startswith(("sympy", "pint")):
            return None
        raise


def _answer(tree, glossary, units):
    tokens = _tokens(tree)
    if not tokens or not (tokens[-1][0] == "sym" and tokens[-1][1] == "=" and tokens[-1][2] is None):
        return None
    tokens = tokens[:-1]
    statements = [s for s, _ in _split(tokens, {","})]
    last = statements[-1]
    # "ecuación, x ="
    if len(statements) >= 2 and len(last) == 1 and last[0][0] == "sym":
        var = _Parser([], glossary)._symbol(last[0])
        eq_parts = _split(statements[-2], RELATIONS)
        if len(eq_parts) != 2 or eq_parts[0][1] != "=":
            return None
        lhs = _Parser(eq_parts[0][0], glossary).full()
        rhs = _Parser(eq_parts[1][0], glossary).full()
        return _solve(sp.Eq(lhs, rhs), var)
    parts = _split(last, RELATIONS)
    if len(parts) == 2 and parts[0][1] == "=":
        # "ecuación =": se despeja la única incógnita
        lhs = _Parser(parts[0][0], glossary).full()
        rhs = _Parser(parts[1][0], glossary).full()
        free = (lhs - rhs).free_symbols
        if len(free) != 1:
            return None
        return _solve(sp.Eq(lhs, rhs), free.pop())
    if len(parts) != 1:
        return None
    expr = _Parser(last, glossary).full()
    if units:
        return _with_units(expr, units)
    has_calculus = expr.has(sp.Derivative, sp.Integral, sp.Sum, sp.Product, sp.Limit)
    value = expr.doit() if has_calculus else expr
    if not value.free_symbols:
        exact = sp.simplify(value)
        if exact.is_Rational or sp.count_ops(exact) > 3:
            value = _format_number(exact)
            return {"latex": sp.latex(value), "text": str(value), "kind": "value"}
        # irracional corto (π, √2, ln 2): exacto y aproximado
        approx = sp.Float(sp.N(exact, 4), 4)
        return {"latex": f"{sp.latex(exact)} \\approx {sp.latex(approx)}", "text": f"{exact} ≈ {approx}",
                "kind": "value"}
    if isinstance(expr, sp.Integral) and len(expr.limits[0]) == 1:
        return {"latex": sp.latex(value) + " + C", "text": f"{value} + C", "kind": "antiderivative"}
    if has_calculus:
        return {"latex": sp.latex(sp.simplify(value)), "text": str(sp.simplify(value)), "kind": "calculus"}
    # SymPy ya agrupa términos al construir la expresión (2a+3a -> 5a), así que se compara con lo escrito
    simple = sp.simplify(expr)
    if len(visible_tokens(sp.latex(simple))) < len(_written_symbols(last)):
        return {"latex": sp.latex(simple), "text": str(simple), "kind": "simplified"}
    return None


def _written_symbols(tokens):
    """Símbolos escritos de una fila de fichas (contando exponentes, subíndices, fracciones y raíces)."""

    def count(node):
        kind = node[0]
        if kind == "sym":
            return 1
        if kind == "row":
            return sum(count(c) for c in node[1])
        return sum(count(c) for c in node[1:] if isinstance(c, tuple)) + (1 if kind in ("frac", "sqrt") else 0)

    n = 0
    for t in tokens:
        if t[0] == "num":
            n += len(t[1])
        elif t[0] == "sym":
            n += 1
        else:
            n += count(t[1])
        n += sum(count(x) for x in (t[2], t[3]) if x is not None)
    return [None] * n


def _solve(eq, var):
    sols = sp.solve(eq, var)
    if not sols:
        return {"latex": r"\text{sin solución}", "text": "sin solución", "kind": "solve"}
    sols = [_format_number(s) if not s.free_symbols else s for s in sols]
    name = sp.latex(var)
    latex = ",\\; ".join(f"{name} = {sp.latex(s)}" for s in sols)
    return {"latex": latex, "text": ", ".join(f"{var} = {s}" for s in sols), "kind": "solve"}


def _with_units(expr, units):
    """Calcula con unidades: cada símbolo con unidad se sustituye por una cantidad de pint."""
    free = sorted(expr.free_symbols, key=lambda s: s.name)
    if any(s.name not in units for s in free):
        return None
    f = sp.lambdify(free, expr, modules=[{"sqrt": lambda q: q**0.5}, "math"])
    q = f(*[units[s.name] for s in free])
    q = q.to_compact() if hasattr(q, "to_compact") else q
    mag = sp.Float(q.magnitude, 4) if hasattr(q, "magnitude") else sp.Float(q, 4)
    unit = f" {q.units:~P}" if hasattr(q, "units") and str(q.units) != "dimensionless" else ""
    return {"latex": sp.latex(mag) + (r"\," + rf"\mathrm{{{unit.strip()}}}" if unit else ""),
            "text": f"{mag}{unit}", "kind": "value"}
