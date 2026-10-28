# SPDX-License-Identifier: GPL-3.0-or-later
"""Programa auxiliar de OnePepe: lee peticiones JSON (una por línea) en la entrada estándar y responde por la salida.

Peticiones:
  {"id": 1, "cmd": "ping"}
  {"id": 2, "cmd": "recognize", "strokes": [[[x, y], ...], ...], "known": [{"latex", "box"}],
   "notebook_dir": "/ruta", "threshold": 0.95, "clean": true, "equals": true}
  {"id": 3, "cmd": "correct", "strokes": [[[x, y], ...]], "latex": "K", "notebook_dir": "/ruta"}
  {"id": 4, "cmd": "glyph", "latex": "K", "box": [x0, y0, x1, y1]}
  {"id": 5, "cmd": "examples"}                       tus ejemplos guardados (sin trazos)
  {"id": 6, "cmd": "forget", "example": "id"}        borra un ejemplo
  {"id": 7, "cmd": "relabel", "example": "id", "latex": "K"}   cambia su etiqueta

Cada respuesta lleva el mismo "id". Los errores se responden con {"id", "error": "..."}; nunca se cae el programa.
"""

import json
import sys

from . import glossary as gl
from . import glyphs


class Server:
    def __init__(self, classifier=None, personal=None):
        if classifier is None:
            from .classifier import SymbolClassifier

            classifier = SymbolClassifier()
        if personal is None:
            from .personal import PersonalModel

            personal = PersonalModel(classifier)
        self.classifier = classifier
        self.personal = personal

    def handle(self, req):
        cmd = req.get("cmd")
        if cmd == "ping":
            return {"ok": True, "personal_examples": len(self.personal)}
        if cmd == "glyph":
            lines = glyphs.place_glyph(req["latex"], tuple(req["box"]))
            return {"clean": lines}
        if cmd == "recognize":
            from .recognizer import recognize

            path = gl.find_glossary(req.get("notebook_dir"))
            names, values = gl.load(path)
            strokes = [[(float(p[0]), float(p[1])) for p in s] for s in req.get("strokes", []) if s]
            return recognize(
                strokes,
                self.classifier,
                personal=self.personal,
                known=req.get("known", []),
                glossary_names=names,
                units=gl.units(values),
                threshold=float(req.get("threshold", 0.95)),
                clean=bool(req.get("clean", True)),
                equals=bool(req.get("equals", True)),
            )
        if cmd == "correct":
            strokes = [[(float(p[0]), float(p[1])) for p in s] for s in req["strokes"] if s]
            self.personal.add_correction(
                req["latex"], strokes, predicted=req.get("predicted"), notebook=req.get("notebook_dir")
            )
            notebook_dir = req.get("notebook_dir")
            if notebook_dir and req.get("add_to_glossary", True):
                path = gl.find_glossary(notebook_dir)
                if path:
                    gl.add_name(path, req["latex"])
            from .segment import bbox

            return {"ok": True, "clean": glyphs.place_glyph(req["latex"], bbox(strokes))}
        if cmd == "examples":
            return {"examples": self.personal.list_examples()}
        if cmd == "forget":
            return {"ok": self.personal.forget(req["example"])}
        if cmd == "relabel":
            return {"ok": self.personal.relabel(req["example"], req["latex"])}
        return {"error": f"orden desconocida: {cmd}"}

    def serve(self, inp=sys.stdin, out=sys.stdout):
        for line in inp:
            line = line.strip()
            if not line:
                continue
            req_id = None
            try:
                req = json.loads(line)
                req_id = req.get("id")
                resp = self.handle(req)
            except Exception as e:  # noqa: BLE001 - una petición rara no debe tumbar el programa
                resp = {"error": f"{type(e).__name__}: {e}"}
            resp["id"] = req_id
            out.write(json.dumps(resp, ensure_ascii=False) + "\n")
            out.flush()


def main():
    Server().serve()


if __name__ == "__main__":
    main()
