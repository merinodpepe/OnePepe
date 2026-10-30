# SPDX-License-Identifier: GPL-3.0-or-later
"""Modo entreno: escribir a mano expresiones conocidas para enseñar tu letra al reconocedor.

La app muestra una expresión (dibujada desde su LaTeX) y un recuadro; tú la escribes y pasas a la siguiente. Como se
sabe exactamente qué has escrito, cada expresión se convierte en ejemplos de tus símbolos sin adivinar renglones.

- Los **sets** (listas de expresiones, sin datos tuyos) están en `tools/onepepe-math/entrenos/*.json`.
- La **sesión** es un archivo `*.entreno.json` que guarda la app (no es un `.rnote`): el set, por dónde vas y tus
  trazos con sus tiempos. Formato en `new_session`.
- **Importar** una sesión corta cada expresión en símbolos con el corte forzado (`forced.py`) y los guarda en tu
  almacén local, como `importar_mi_letra.py`. Después se reentrena desde la app.
"""

import base64
import io
import os
import re
import time
from functools import lru_cache

from .forced import forced_groups, junk_examples
from .latex_tokens import _INVISIBLE, _TOKEN
from .layout import FUNCTION_ALIASES, FUNCTIONS
from .symbols import normalize_label

SETS_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "entrenos")
SESSION_VERSION = 1


# ---------- lo que se escribe ----------


def expected_symbols(latex):
    """Los símbolos que se escriben a mano en una expresión: las funciones letra a letra, sin espacios ni llaves."""
    latex = re.sub(r"\\(begin|end)\{[a-z]*\}", " ", latex)
    latex = latex.replace(r"\mathrm", "").replace(r"\,", " ").replace(r"\quad", " ").replace(r"\ ", " ")
    out = []
    for t in _TOKEN.findall(latex):
        if t in _INVISIBLE or t in "{}":
            continue
        name = t[1:] if t.startswith("\\") else None
        if name and (name in FUNCTIONS or name in FUNCTION_ALIASES):
            out += list(name)
            continue
        out.append(normalize_label(t))
    return out


# ---------- sets y sesiones ----------


def list_sets(directory=SETS_DIR):
    """Los sets disponibles: [{"name", "title", "description", "count"}]."""
    import json

    out = []
    for f in sorted(os.listdir(directory)):
        if f.endswith(".json"):
            with open(os.path.join(directory, f), encoding="utf-8") as fh:
                data = json.load(fh)
            out.append({"name": data["name"], "title": data["title"], "description": data.get("description", ""),
                        "count": len(data["items"])})
    return out


def load_set(name, directory=SETS_DIR):
    import json

    with open(os.path.join(directory, f"{name}.json"), encoding="utf-8") as fh:
        return json.load(fh)


def new_session(name, directory=SETS_DIR):
    """Una sesión nueva de un set: lo que la app guarda en el archivo `*.entreno.json`.

    {"format": "onepepe-entreno", "version": 1, "set": nombre, "title", "created", "current": 0,
     "items": [{"id", "kind": "expr" | "steps" | "symbols", "lines": [latex, ...], "hint"}],
     "answers": {id: {"strokes": [[[x, y], ...]], "times": [[t0, t1]], "size": [ancho, alto], "lines": n,
                      "line_height": alto de cada renglón guía,
                      "written": fecha, "skipped": bool, "imported": bool}}}

    La app añade además "app_version" al guardar.
    """
    data = load_set(name, directory)
    return {
        "format": "onepepe-entreno",
        "version": SESSION_VERSION,
        "set": data["name"],
        "title": data["title"],
        "created": time.strftime("%Y-%m-%d %H:%M"),
        "current": 0,
        "items": data["items"],
        "answers": {},
    }


# ---------- dibujar la expresión ----------


@lru_cache(maxsize=512)
def render(lines, color="#000000", dpi=140):
    """PNG (en base64) de la expresión dibujada con matplotlib, un renglón debajo de otro, en `color` sobre fondo
    transparente (para el tema claro y el oscuro). None si matplotlib no está o no la entiende."""
    if isinstance(lines, str):
        lines = (lines,)
    try:
        import matplotlib

        matplotlib.use("Agg")
        from matplotlib.figure import Figure
    except ImportError:
        return None
    fig = Figure(figsize=(0.01, 0.01))
    text = "\n".join(f"${line}$" for line in lines)
    fig.text(0, 0, text, fontsize=22, color=color, linespacing=1.9)
    buf = io.BytesIO()
    try:
        fig.savefig(buf, dpi=dpi, format="png", transparent=True, bbox_inches="tight", pad_inches=0.05)
    except Exception:  # noqa: BLE001 - una expresión que no sabe dibujar se enseña como texto
        return None
    return base64.b64encode(buf.getvalue()).decode("ascii")


# ---------- importar ----------


def _split_by_guides(strokes, size, n_lines, line_height=None):
    """Reparte los trazos entre los renglones guía del recuadro (bandas de `line_height`, o de alto igual si no se
    sabe) por su centro vertical."""
    if n_lines <= 1:
        return [list(range(len(strokes)))]
    band = line_height or size[1] / n_lines
    out = [[] for _ in range(n_lines)]
    for i, s in enumerate(strokes):
        cy = (min(p[1] for p in s) + max(p[1] for p in s)) / 2
        out[min(max(int(cy // band), 0), n_lines - 1)].append(i)
    return out


def import_session(session, classifier, store, source=None):
    """Convierte lo escrito en ejemplos de tu letra (símbolos y «trozos») en el almacén.

    Salta lo ya importado y lo que no encaja (más o menos trazos de los que caben). Devuelve
    {"imported": [ids], "failed": [ids], "symbols": n, "junk": n}.
    """
    source = source or f"entreno:{session.get('set', '')}"
    items = {it["id"]: it for it in session.get("items", [])}
    out = {"imported": [], "failed": [], "symbols": 0, "junk": 0}
    for item_id, answer in session.get("answers", {}).items():
        item = items.get(item_id)
        if item is None or answer.get("skipped") or answer.get("imported") or not answer.get("strokes"):
            continue
        strokes = [[(float(p[0]), float(p[1])) for p in s] for s in answer["strokes"] if s]
        lines = item["lines"]
        groups = _split_by_guides(strokes, answer.get("size", [1.0, 1.0]), len(lines), answer.get("line_height"))
        labeled_lines = []
        for latex, idx in zip(lines, groups):
            line_strokes = [strokes[i] for i in idx]
            labeled = forced_groups(line_strokes, expected_symbols(latex), classifier)
            if labeled is None:
                labeled_lines = None
                break
            labeled_lines.append((line_strokes, labeled))
        if labeled_lines is None:
            out["failed"].append(item_id)
            continue
        for line_strokes, labeled in labeled_lines:
            for group, label in labeled:
                store.add_correction(label, [line_strokes[i] for i in group], notebook=source)
                out["symbols"] += 1
            for piece in junk_examples(line_strokes, labeled):
                store.add_correction("__trozo__", piece, notebook=source, kind="junk")
                out["junk"] += 1
        out["imported"].append(item_id)
    return out
