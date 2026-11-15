# SPDX-License-Identifier: GPL-3.0-or-later
"""Todo lo que se calcula con los renglones ya reconocidos de una página (orden `page` del programa auxiliar).

Cada vez que cambia algo (un símbolo nuevo, una corrección, una opción), la app envía los renglones de la página
con sus símbolos y recibe lo que hay que mostrar al lado de cada uno:

- el resultado del «=» final (y un «?» con lo que se ha leído cuando no se puede calcular);
- variables: `x = 3` define x en la página; los renglones que la usan muestran su valor (fase 3 de MyScript);
- revisión de pasos: cada paso de un desarrollo se compara con el anterior y, si no es equivalente, se marca y se
  explica el error típico que lo produce (fase 4);
- el LaTeX de cada renglón (fase 5) y las formas de una fracción: fracción, decimal y mixta (fase 6).

Nada de esto cambia la tinta: son anotaciones que la app dibuja encima.
"""

import random
import re
from dataclasses import replace

import sympy as sp

from .entorno import Entorno, _value, definir
from .evaluate import (
    RELATIONS,
    ParseError,
    _natural_log_names,
    _parse,
    _Parser,
    _split,
    _tokens,
    answer,
)
from .layout import Sym, parse, to_latex
from .symbols import STEP_SEPARATORS


# ---------- utilidades ----------


def _row_syms(row):
    return [Sym(s["latex"], tuple(s["box"]), tuple(s.get("alts", ()))) for s in row.get("symbols", [])]


def _bounds(syms):
    return (
        min(s.box[0] for s in syms),
        min(s.box[1] for s in syms),
        max(s.box[2] for s in syms),
        max(s.box[3] for s in syms),
    )


def _typical(syms):
    hs = sorted(s.h for s in syms if s.label not in ("-", "=", ".", ",", r"\cdot", r"\frac"))
    return hs[len(hs) // 2] if hs else max((s.w for s in syms), default=10.0)


def _ends_with_equals(tokens):
    return bool(tokens) and tokens[-1][0] == "sym" and tokens[-1][1] == "=" and tokens[-1][2] is None


def _expr(tokens, vista):
    """Expresión de SymPy de unas fichas, tal como se escribió (sin simplificar: hace falta para explicar errores),
    con lo que vale en `vista`."""
    if not tokens:
        raise ParseError("vacío")
    with sp.evaluate(False):
        return _parse(tokens, {}, vista)[0]


# ---------- formas de un número (fase 6) ----------


def number_formats(value):
    """Formas de mostrar un número racional: [fracción, decimal, mixta]. Vacío si no es una fracción."""
    try:
        value = sp.nsimplify(value) if isinstance(value, sp.Float) else sp.Rational(value)
    except (TypeError, ValueError):
        return []
    if not isinstance(value, sp.Rational) or value.q == 1 or value.q > 1000:
        return []
    out = [{"text": f"{value.p}/{value.q}", "latex": sp.latex(value), "kind": "fraction"}]
    dec = sp.Float(value, 6)
    text = f"{float(dec):.6g}"
    out.append({"text": text, "latex": text, "kind": "decimal"})
    if abs(value) > 1:
        whole = int(abs(value.p) // value.q)
        rest = abs(value) - whole
        sign = "-" if value < 0 else ""
        out.append(
            {
                "text": f"{sign}{whole} {rest.p}/{rest.q}",
                "latex": f"{sign}{whole}\\tfrac{{{rest.p}}}{{{rest.q}}}",
                "kind": "mixed",
            }
        )
    return out


# ---------- caché por renglón ----------

_CACHE = {}


def _cache(clave, calcula):
    """Lo que se calcula de un renglón depende solo de sus fichas, las opciones y lo que vale; no de dónde está."""
    if clave not in _CACHE:
        if len(_CACHE) >= 512:
            _CACHE.clear()
        _CACHE[clave] = calcula()
    return _CACHE[clave]


# ---------- revisión de pasos (fase 4) ----------


def _numeric(expr, values):
    v = complex(expr.subs(values).doit().evalf())
    return v


def _random_points(symbols, n=6, seed=7):
    rnd = random.Random(seed)
    return [{s: sp.Rational(rnd.randint(55, 245), 100) for s in symbols} for _ in range(n)]


def same_expr(a, b):
    """¿Valen lo mismo dos expresiones para muchos valores al azar de sus variables?"""
    symbols = sorted(a.free_symbols | b.free_symbols, key=lambda s: s.name)
    ok = 0
    for point in _random_points(symbols):
        try:
            va, vb = _numeric(a, point), _numeric(b, point)
        except (TypeError, ValueError, ZeroDivisionError):
            continue
        if any(map(lambda z: z != z or abs(z) == float("inf"), (va, vb))):
            continue
        if abs(va - vb) > 1e-7 * (1 + abs(va)):
            return False
        ok += 1
    return ok > 0


def same_equation(e1, e2):
    """¿Tienen las mismas soluciones dos ecuaciones (lhs, rhs)?"""
    f1 = e1[0] - e1[1]
    f2 = e2[0] - e2[1]
    symbols = sorted(f1.free_symbols | f2.free_symbols, key=lambda s: s.name)
    if len(symbols) == 1:
        x = symbols[0]
        try:
            s1 = set(sp.solve(sp.Eq(f1.doit(), 0), x))
            s2 = set(sp.solve(sp.Eq(f2.doit(), 0), x))
            if s1 or s2:
                return {sp.nsimplify(v) for v in s1} == {sp.nsimplify(v) for v in s2}
        except Exception:  # noqa: BLE001 - si no se puede despejar, se compara de otra forma
            pass
    # proporcionales: f1 = k·f2 con k constante
    ratios = []
    for point in _random_points(symbols):
        try:
            v2 = _numeric(f2, point)
            if abs(v2) < 1e-9:
                continue
            ratios.append(_numeric(f1, point) / v2)
        except (TypeError, ValueError, ZeroDivisionError):
            continue
    return bool(ratios) and all(abs(r - ratios[0]) < 1e-7 * (1 + abs(ratios[0])) for r in ratios)


def _subexprs(expr, cls):
    return [e for e in sp.preorder_traversal(expr) if isinstance(e, cls)]


def _expression_mistakes(a):
    """Versiones de `a` con un error típico: [(expresión equivocada, explicación)]."""
    out = []
    with sp.evaluate(False):
        terms = sp.Add.make_args(a)
        if len(terms) > 1:
            for t in terms:
                out.append((sp.Add(*[(-u if u is t else u) for u in terms]), "Se ha perdido un signo: revisa el signo de cada término."))
        for m in _subexprs(a, sp.Mul):
            factors = m.args
            for k, f in enumerate(factors):
                if isinstance(f, sp.Add) and len(f.args) > 1:
                    others = sp.Mul(*[g for j, g in enumerate(factors) if j != k])
                    wrong = sp.Add(sp.Mul(others, f.args[0]), *f.args[1:])
                    out.append((a.xreplace({m: wrong}), "Falta multiplicar el factor por todos los términos del paréntesis."))
        for p in _subexprs(a, sp.Pow):
            base, exp = p.args
            if isinstance(base, sp.Add) and exp.is_Integer and exp > 1:
                wrong = sp.Add(*[sp.Pow(t, exp) for t in base.args])
                out.append((a.xreplace({p: wrong}), "Una potencia no se reparte en una suma: (a+b)² = a² + 2ab + b²."))
            if isinstance(base, sp.Add) and exp == sp.Rational(1, 2):
                wrong = sp.Add(*[sp.sqrt(t) for t in base.args])
                out.append((a.xreplace({p: wrong}), "La raíz no se reparte en una suma: √(a+b) ≠ √a + √b."))
        for s in _subexprs(a, sp.Add):
            fracs = [t for t in s.args if isinstance(t, sp.Mul) and any(isinstance(g, sp.Pow) and g.exp == -1 for g in t.args)]
            if len(fracs) >= 2:
                nums, dens = [], []
                for t in fracs[:2]:
                    n, d = sp.fraction(t)
                    nums.append(n)
                    dens.append(d)
                wrong = sp.Add(sp.Mul(sp.Add(*nums), sp.Pow(sp.Add(*dens), -1)), *[t for t in s.args if t not in fracs[:2]])
                out.append((a.xreplace({s: wrong}), "Las fracciones no se suman sumando numeradores y denominadores: busca el denominador común."))
    return out


def _equation_mistakes(lhs, rhs):
    """Versiones de la ecuación lhs = rhs con un error típico."""
    out = []
    with sp.evaluate(False):
        for side, other, left in ((lhs, rhs, True), (rhs, lhs, False)):
            terms = sp.Add.make_args(side)
            if len(terms) > 1:
                for t in terms:
                    rest = sp.Add(*[u for u in terms if u is not t])
                    moved = sp.Add(other, t)
                    eq = (rest, moved) if left else (moved, rest)
                    out.append((eq, "Has pasado un término al otro lado sin cambiarle el signo."))
        # a·x = b  →  x = b − a, x = b·a o x = a/b
        if isinstance(lhs, sp.Mul):
            numbers = [f for f in lhs.args if f.is_number]
            rest = [f for f in lhs.args if not f.is_number]
            if len(numbers) == 1 and rest:
                a, x = numbers[0], sp.Mul(*rest)
                out.append(((x, sp.Add(rhs, -a)), "El número que multiplica pasa al otro lado dividiendo, no restando."))
                out.append(((x, sp.Mul(rhs, a)), "El número que multiplica pasa al otro lado dividiendo, no multiplicando."))
                out.append(((x, sp.Mul(a, sp.Pow(rhs, -1))), "Al despejar, la división va al revés."))
    for wrong, why in _expression_mistakes(lhs):
        out.append(((wrong, rhs), why))
    for wrong, why in _expression_mistakes(rhs):
        out.append(((lhs, wrong), why))
    return out


def explain_expressions(a, b):
    for wrong, why in _expression_mistakes(a):
        try:
            if same_expr(wrong, b):
                return why
        except Exception:  # noqa: BLE001
            continue
    return "No es equivalente al paso anterior."


def explain_equations(e1, e2):
    for wrong, why in _equation_mistakes(*e1):
        try:
            if same_equation(wrong, e2):
                return why
        except Exception:  # noqa: BLE001
            continue
    return "No tiene las mismas soluciones que el paso anterior."


def _statement(tokens, vista):
    """("eq", (lhs, rhs)) o ("expr", e) o None."""
    parts = _split(tokens, RELATIONS)
    if len(parts) == 1:
        return ("expr", _expr(tokens, vista))
    if len(parts) == 2 and parts[0][1] == "=":
        return ("eq", (_expr(parts[0][0], vista), _expr(parts[1][0], vista)))
    return None


def check_steps(a, b):
    """Compara dos pasos. None si no se pueden comparar; "" si están bien; si no, la explicación."""
    if a is None or b is None or a[0] != b[0]:
        return None
    if a[0] == "expr":
        return "" if same_expr(a[1], b[1]) else explain_expressions(a[1], b[1])
    if not (a[1][0] - a[1][1]).free_symbols & (b[1][0] - b[1][1]).free_symbols:
        return None
    return "" if same_equation(a[1], b[1]) else explain_equations(a[1], b[1])


# ---------- la página ----------


def decimal_comma(text, latex=False):
    """Los números con coma decimal, como los escribe el usuario: 0.6931 → 0,6931 (en LaTeX, 0{,}6931)."""
    return re.sub(r"(\d)\.(\d)", r"\1{,}\2" if latex else r"\1,\2", text)


def _mark(kind, text, latex, anchor, height, **extra):
    out = {"kind": kind, "text": decimal_comma(text), "latex": decimal_comma(latex, latex=True),
           "anchor": [round(anchor[0], 2), round(anchor[1], 2)], "height": round(height, 2)}
    if "formats" in extra:
        extra["formats"] = [dict(f, text=decimal_comma(f["text"]), latex=decimal_comma(f["latex"], latex=True))
                            for f in extra["formats"]]
    out.update(extra)
    return out


def _after(box, typical):
    return (box[2] + 0.3 * max(typical, 8.0), (box[1] + box[3]) / 2)


def _resultado(tree, units, vista):
    result = answer(tree, units=units or None, vista=vista)
    if result is None and units:
        result = answer(tree, vista=vista)
    return result


def _valor_variable(tree, tokens, d, vista, units):
    """{"text", "latex"} del valor de un renglón sin «=» que usa variables (o de una definición `y = 2x + 3`), si
    todo lo que usa tiene valor."""
    if d is None:
        if len(_split(tokens, RELATIONS | {","} | STEP_SEPARATORS)) != 1:
            return None
        expr = _Parser(tokens, {}, vista.funciones, vista.conocidas).full()
    elif vista.cantidades:
        return None
    else:
        expr = d.expr
    resuelta, cambiado = vista.resolver(expr)
    libres = {s.name for s in resuelta.free_symbols}
    if not cambiado or not (not libres or libres <= set(vista.cantidades)):
        return None
    if vista.cantidades:
        return answer(("row", tree[1] + [("sym", "=")]), units=units, vista=vista)
    value = _value(resuelta)
    value = value if value.is_Rational else sp.Float(value, 4)
    return _natural_log_names({"text": str(value), "latex": sp.latex(value), "kind": "value"})


def page(rows, equals=True, variables=False, check=False, glossary_units=None, umbral_renglon=0.7):
    """Anotaciones de cada renglón. `rows`: [{"symbols": [{"latex", "box"}]}], de arriba abajo."""
    glossary_units = glossary_units or {}
    parsed = []
    for row in rows:
        syms = _row_syms(row)
        if not syms:
            parsed.append(None)
            continue
        tree = parse(syms)
        parsed.append((syms, tree, _tokens(tree)))
    # definiciones de la página
    definiciones = []
    entorno = Entorno(definiciones)
    if variables:
        for i, p in enumerate(parsed):
            if p is None or _ends_with_equals(p[2]):
                continue
            vista = entorno.vista(i)
            d = _cache(("d", repr(p[2]), vista.clave()), lambda: definir(p[2], i, vista, glossary_units))
            if d:
                definiciones.append(replace(d, renglon=i, _texto=None))
    por_renglon = {d.renglon: d for d in definiciones}
    out = []
    previous = previous_d = None
    for i, p in enumerate(parsed):
        if p is None:
            out.append({"latex": "", "marks": []})
            previous = previous_d = None
            continue
        syms, tree, tokens = p
        typical = _typical(syms)
        box = _bounds(syms)
        marks = []
        vista = entorno.vista(i)
        units = {**glossary_units, **vista.cantidades}
        clave = (repr(tokens), repr(sorted((k, str(v)) for k, v in units.items())), vista.clave())
        d = por_renglon.get(i)
        # 1. el «=» final
        if equals and _ends_with_equals(tokens):
            eq = max((s for s in syms if s.label == "="), key=lambda s: s.box[2])
            anchor = _after(eq.box, typical)
            height = max(eq.w, eq.h, 0.8 * typical) * 1.2
            result = _cache(("=",) + clave, lambda: _resultado(tree, units, vista))
            if result is None:
                marks.append(_mark("unknown", "?", to_latex(tree), anchor, height, at=syms.index(eq)))
            else:
                formats = []
                if result["kind"] == "value":
                    try:
                        formats = number_formats(sp.sympify(result["text"]))
                    except (sp.SympifyError, TypeError, ValueError):
                        formats = []
                marks.append(_mark(result["kind"], result["text"], result["latex"], anchor, height, formats=formats,
                                   at=syms.index(eq)))
        # 2. variables: una expresión que las usa (o una definición `y = 2x + 3`) muestra su valor
        elif variables and len(tokens) >= 2 and (d is None or d.tipo == "expr"):

            def calcula():
                try:
                    return _valor_variable(tree, tokens, d, vista, units)
                except Exception:  # noqa: BLE001 - un renglón que no se entiende no anota nada
                    return None

            result = _cache(("v",) + clave, calcula)
            if result:
                formats = number_formats(sp.sympify(result["text"])) if not vista.cantidades else []
                for f in formats:
                    f["text"] = "= " + f["text"]
                marks.append(_mark("variable", "= " + result["text"], "= " + result["latex"], _after(box, typical),
                                   0.9 * typical, formats=formats))
        # 3. revisión de pasos: solo valen como datos las definiciones anteriores a este renglón y al de encima
        if check:
            # dos definiciones seguidas de nombres distintos (x = 5, y = 2x + 3) no son pasos de un desarrollo
            otro = d is not None and previous_d is not None and d.nombre != previous_d.nombre
            marks += _check_row(tokens, typical, box, None if otro else previous, entorno.vista(i - 1, solo_encima=True))
        # M8: dudosa si algún símbolo del renglón tiene p baja (sin p en la petición, segura)
        sure = min([s["p"] for s in rows[i]["symbols"] if "p" in s] + [1.0]) >= umbral_renglon
        for m in marks:
            m["sure"] = sure
        out.append({"latex": to_latex(tree), "marks": marks})
        previous = (tokens, syms)
        previous_d = d
    return out


def _check_row(tokens, typical, box, previous, vista):
    """Pasos dentro del renglón (separados por → ⇒ ;, o una cadena A = B = C) y con el renglón de encima."""
    why = _cache(("c", repr(tokens), repr(previous[0]) if previous else None, vista.clave()),
                 lambda: _check_why(tokens, previous, vista))
    if not why:
        return []
    # debajo del renglón, empezando por la izquierda: así no se cruza con el resultado del «=» ni con el valor
    # de las variables, que van a la derecha
    return [_mark("error", "✗ " + why, why, (box[0], box[3] + 0.55 * typical), 0.55 * typical)]


def _check_why(tokens, previous, vista):
    """La explicación del primer paso que no cuadra, o "" si todo cuadra (o no se entiende)."""
    if _ends_with_equals(tokens):
        tokens = tokens[:-1]
    # «= …» sigue la expresión del renglón de encima
    continues = bool(tokens) and tokens[0][0] == "sym" and tokens[0][1] == "="
    body = tokens[1:] if continues else tokens
    try:
        steps = [t for t, _ in _split(body, STEP_SEPARATORS) if t]
        stmts = [_statement(t, vista) for t in steps]
        for a, b in zip(stmts, stmts[1:]):
            why = check_steps(a, b)
            if why:
                return why
        # cadena «A = B = C»: cada parte igual a la anterior
        parts = [t for t, _ in _split(body, {"="})]
        if len(parts) >= 3 and all(parts):
            exprs = [_expr(t, vista) for t in parts]
            for a, b in zip(exprs, exprs[1:]):
                if not same_expr(a, b):
                    return explain_expressions(a, b)
        # con el renglón de encima
        if previous is not None:
            prev_tokens = previous[0][:-1] if _ends_with_equals(previous[0]) else previous[0]
            prev_steps = [t for t, _ in _split(prev_tokens, STEP_SEPARATORS) if t]
            if not prev_steps:
                return ""
            if continues:
                a = _expr(_split(prev_steps[-1], {"="})[-1][0], vista)
                b = _expr(_split(body, {"="})[0][0], vista)
                if not same_expr(a, b):
                    return explain_expressions(a, b)
            elif stmts:
                return check_steps(_statement(prev_steps[-1], vista), stmts[0]) or ""
    except Exception:  # noqa: BLE001 - lo que no se entiende no se marca
        return ""
    return ""


# ---------- LaTeX de varios renglones (fase 5) ----------


def latex_rows(rows):
    out = []
    for row in rows:
        syms = _row_syms(row)
        if syms:
            out.append(to_latex(parse(syms)))
    return out
