# SPDX-License-Identifier: GPL-3.0-or-later
"""Lectura de archivos InkML (formato del dataset MathWriting)."""

import xml.etree.ElementTree as ET

_NS = "{http://www.w3.org/2003/InkML}"


def read_inkml(path):
    """Devuelve (anotaciones: dict, trazos: lista de listas de (x, y))."""
    root = ET.parse(path).getroot()
    annotations = {a.get("type"): (a.text or "") for a in root.iter(_NS + "annotation")}
    strokes = []
    for trace in root.iter(_NS + "trace"):
        pts = []
        for item in (trace.text or "").split(","):
            vals = item.split()
            if len(vals) >= 2:
                pts.append((float(vals[0]), float(vals[1])))
        if pts:
            strokes.append(pts)
    return annotations, strokes
