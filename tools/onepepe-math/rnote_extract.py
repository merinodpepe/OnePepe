#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""OnePepe: extrae los trazos de un archivo .rnote y los convierte en datos de prueba.

Un .rnote es un JSON comprimido con gzip. Este programa:
  1. lee los trazos de tinta (pincel) en el orden en que se dibujaron;
  2. los agrupa en renglones (una expresión por renglón);
  3. escribe un JSON con los puntos de cada trazo y, si se pide, una imagen SVG por renglón
     para comprobar que la separación es correcta;
  4. si se le da un archivo de expresiones (una por línea, en LaTeX), asocia cada renglón
     a su expresión, para tener la "respuesta correcta" de cada muestra.

Solo usa la biblioteca estándar de Python. Nada se envía a ningún sitio.

Uso:
    python3 rnote_extract.py mis_expresiones.rnote -o datos.json --svg carpeta_svg \
        --labels expresiones.txt
"""

import argparse
import gzip
import json
import os
import sys


def load_rnote(path):
    """Devuelve el engine_snapshot de un .rnote como diccionario."""
    with open(path, "rb") as f:
        raw = f.read()
    try:
        data = json.loads(gzip.decompress(raw))
    except OSError:
        # por si alguien pasa el JSON ya descomprimido
        data = json.loads(raw)
    version = data.get("version", "?")
    snapshot = data.get("data", {}).get("engine_snapshot")
    if snapshot is None:
        raise ValueError(
            f"{path}: no encuentro 'data.engine_snapshot' (versión del archivo {version}). "
            "Este extractor está hecho para el formato de Rnote 0.15."
        )
    return version, snapshot


def path_points(path):
    """Puntos (x, y, presión) de un PenPath: el inicio y el final de cada segmento."""
    pts = []

    def add(el):
        x, y = el["pos"]
        pts.append([round(float(x), 3), round(float(y), 3), round(float(el.get("pressure", 0.5)), 3)])

    add(path["start"])
    for seg in path.get("segments", []):
        # lineto / quadbezto / cubbezto: todos tienen un "end"
        (kind, body), = seg.items()
        add(body["end"])
    return pts


def extract_strokes(snapshot):
    """Lista de trazos de pincel ordenados por el orden en que se dibujaron."""
    strokes = []
    chrono = snapshot.get("chrono_components", [])
    for idx, comp in enumerate(snapshot.get("stroke_components", [])):
        value = comp.get("value") if isinstance(comp, dict) else None
        if not value or "brushstroke" not in value:
            # huecos, formas, textos, imágenes: no son tinta a mano
            continue
        order = idx
        if idx < len(chrono) and isinstance(chrono[idx], dict) and chrono[idx].get("value"):
            order = chrono[idx]["value"].get("t", idx)
        pts = path_points(value["brushstroke"]["path"])
        xs = [p[0] for p in pts]
        ys = [p[1] for p in pts]
        strokes.append(
            {
                "order": order,
                "points": pts,
                "bbox": [min(xs), min(ys), max(xs), max(ys)],
            }
        )
    strokes.sort(key=lambda s: s["order"])
    for i, s in enumerate(strokes):
        s["id"] = i
    return strokes


def merge_close_lines(lines, merge_factor=0.5):
    """Junta renglones separados por un hueco mucho menor que el hueco típico entre expresiones.

    Un exponente alto, una raíz o la raya de una fracción pueden quedar como un "renglón" aparte; el hueco que los
    separa de su expresión es mucho más pequeño que el que hay entre una expresión y la siguiente.
    """
    if len(lines) < 3:
        return lines
    gaps = sorted(b["top"] - a["bottom"] for a, b in zip(lines, lines[1:]))
    limit = merge_factor * gaps[len(gaps) // 2]
    merged = [lines[0]]
    for line in lines[1:]:
        if line["top"] - merged[-1]["bottom"] < limit:
            merged[-1]["strokes"] += line["strokes"]
            merged[-1]["bottom"] = max(merged[-1]["bottom"], line["bottom"])
        else:
            merged.append(line)
    return merged


def split_lines(strokes, gap_factor=0.35, merge_factor=0.5):
    """Agrupa los trazos en renglones según su posición vertical.

    Dos trazos están en el mismo renglón si sus franjas verticales se solapan o están
    separadas menos de `gap_factor` veces la altura típica de un trazo. Después se juntan los renglones
    separados por un hueco mucho menor que el típico entre expresiones (ver `merge_close_lines`).
    """
    if not strokes:
        return []
    heights = sorted(max(s["bbox"][3] - s["bbox"][1], 1.0) for s in strokes)
    typical = heights[len(heights) // 2]
    by_top = sorted(strokes, key=lambda s: (s["bbox"][1] + s["bbox"][3]) / 2)
    lines = []
    for s in by_top:
        top, bottom = s["bbox"][1], s["bbox"][3]
        if lines and top <= lines[-1]["bottom"] + gap_factor * typical:
            lines[-1]["strokes"].append(s)
            lines[-1]["bottom"] = max(lines[-1]["bottom"], bottom)
        else:
            lines.append({"strokes": [s], "top": top, "bottom": bottom})
    lines = merge_close_lines(lines, merge_factor)
    result = []
    for i, line in enumerate(lines):
        # dentro de un renglón se mantiene el orden de escritura
        ss = sorted(line["strokes"], key=lambda s: s["order"])
        result.append(
            {
                "line": i,
                "stroke_ids": [s["id"] for s in ss],
                "bbox": [
                    min(s["bbox"][0] for s in ss),
                    min(s["bbox"][1] for s in ss),
                    max(s["bbox"][2] for s in ss),
                    max(s["bbox"][3] for s in ss),
                ],
            }
        )
    return result


def read_labels(path):
    """Una expresión LaTeX por línea. Se ignoran líneas vacías y las que empiezan por '#'."""
    labels = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#"):
                labels.append(line)
    return labels


def write_svg(path, strokes):
    xs0 = min(s["bbox"][0] for s in strokes) - 10
    ys0 = min(s["bbox"][1] for s in strokes) - 10
    w = max(s["bbox"][2] for s in strokes) - xs0 + 10
    h = max(s["bbox"][3] for s in strokes) - ys0 + 10
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{xs0:.1f} {ys0:.1f} {w:.1f} {h:.1f}" '
        f'width="{w:.0f}" height="{h:.0f}">',
        f'<rect x="{xs0:.1f}" y="{ys0:.1f}" width="{w:.1f}" height="{h:.1f}" fill="white"/>',
    ]
    for s in strokes:
        d = " ".join(f"{p[0]:.1f},{p[1]:.1f}" for p in s["points"])
        out.append(
            f'<polyline points="{d}" fill="none" stroke="black" stroke-width="2" '
            'stroke-linecap="round" stroke-linejoin="round"/>'
        )
    out.append("</svg>")
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(out))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("rnote", help="archivo .rnote")
    ap.add_argument("-o", "--output", help="JSON de salida (por defecto, se escribe en pantalla)")
    ap.add_argument("--svg", metavar="CARPETA", help="escribe un SVG por renglón en esta carpeta")
    ap.add_argument("--labels", metavar="ARCHIVO", help="expresiones en LaTeX, una por renglón, en el mismo orden")
    args = ap.parse_args(argv)

    version, snapshot = load_rnote(args.rnote)
    strokes = extract_strokes(snapshot)
    lines = split_lines(strokes)

    if args.labels:
        labels = read_labels(args.labels)
        if len(labels) != len(lines):
            print(
                f"AVISO: hay {len(lines)} renglones con tinta y {len(labels)} expresiones en {args.labels}. "
                "Se asocian en orden hasta donde llegue; revisa los SVG.",
                file=sys.stderr,
            )
        for line, label in zip(lines, labels):
            line["latex"] = label

    result = {
        "source": os.path.basename(args.rnote),
        "rnote_version": str(version),
        "note": "Los .rnote no guardan la hora de cada punto: 'order' es el orden en que se dibujaron los trazos.",
        "strokes": strokes,
        "lines": lines,
    }

    if args.svg:
        os.makedirs(args.svg, exist_ok=True)
        by_id = {s["id"]: s for s in strokes}
        for line in lines:
            write_svg(
                os.path.join(args.svg, f"renglon_{line['line']:02d}.svg"),
                [by_id[i] for i in line["stroke_ids"]],
            )

    text = json.dumps(result, ensure_ascii=False, indent=1)
    if args.output:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(text)
        print(
            f"{len(strokes)} trazos en {len(lines)} renglones -> {args.output}",
            file=sys.stderr,
        )
    else:
        print(text)


if __name__ == "__main__":
    main()
