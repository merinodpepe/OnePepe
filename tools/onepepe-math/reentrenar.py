#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""OnePepe: reentrenar el reconocedor de símbolos con tu letra (lo usa el botón «Reentrenar» de la app).

  --estimar   dice cuántos ejemplos tuyos hay y cuánto tardará (según lo que tardó la vez anterior), en una línea JSON.
  (nada)      reentrena: escribe el progreso en líneas JSON {"stage", "progress", "message"} y al final
              {"stage": "done", "replaced": bool, "message"}.

Cómo reentrena:
  1. Entrena con todo lo tuyo salvo el examen y la validación (examen.json, ver train_classifier.py), junto con
     MathWriting, con el peso de tu letra guardado en ultimo_entreno.json (30 si no hay).
  2. Compara el modelo nuevo con el actual (ARQUITECTURA-EJECUCION.md §6 M2.6): examen sin empeorar, MathWriting
     reservado sin bajar más de 2 puntos y sin más sustituciones erróneas. Si no, mantiene el actual; el anterior se
     guarda como simbolos.anterior.npz.
  3. Apunta cuánto tardó, para estimar la próxima vez.
Sin modelo actual no hay con qué comparar: el nuevo se usa. Todo ocurre en tu equipo.
"""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time

DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, DIR)

DATA = os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe"))
MATH = os.path.join(DATA, "math")
MODEL = os.path.join(MATH, "simbolos.npz")
PREVIOUS = os.path.join(MATH, "simbolos.anterior.npz")
CORRECTIONS = os.path.join(MATH, "correcciones.jsonl")
LAST = os.path.join(MATH, "ultimo_entreno.json")
DATASET = os.environ.get("ONEPEPE_MATHWRITING", os.path.join(DATA, "mathwriting", "mathwriting-2024"))

DEFAULT_MINUTES = 5.0


def _die_with_parent():
    """En Linux, el entrenamiento se para solo si este proceso muere (también si lo matan sin avisar)."""
    try:
        import ctypes

        ctypes.CDLL("libc.so.6").prctl(1, signal.SIGTERM)  # PR_SET_PDEATHSIG
    except (OSError, AttributeError):
        pass


def emit(**msg):
    print(json.dumps(msg, ensure_ascii=False), flush=True)


def load_examples(path=CORRECTIONS):
    items = []
    try:
        with open(path, encoding="utf-8") as f:
            for line in f:
                try:
                    item = json.loads(line)
                    item["latex"], item["strokes"]
                    items.append(item)
                except (ValueError, KeyError):
                    continue
    except OSError:
        pass
    items.sort(key=lambda it: it.get("date", ""))
    return items


def load_last():
    try:
        with open(LAST, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return {}


def estimate():
    items = load_examples()
    symbols = [it for it in items if it.get("kind", "symbol") != "junk"]
    last = load_last()
    minutes = last.get("seconds", DEFAULT_MINUTES * 60) / 60.0
    return {
        "examples": len(symbols),
        "junk": len(items) - len(symbols),
        "new_since_last": max(len(symbols) - int(last.get("symbols", 0)), 0),
        "minutes": round(minutes, 1),
        "dataset": os.path.isdir(DATASET),
        "model": os.path.exists(MODEL),
    }


def _lineas_examen():
    """(líneas, referencia congelada) de examen_trazos.json, o None si falta."""
    ruta = os.path.join(MATH, "examen_trazos.json")
    if not os.path.exists(ruta):
        return None
    with open(ruta, encoding="utf-8") as f:
        d = json.load(f)
    lineas = [dict(l, strokes=[[tuple(p) for p in s] for s in l["strokes"]],
                   times=[tuple(t) for t in l["times"]] if l.get("times") else None) for l in d.values()]
    return lineas, {k: l["referencia"] for k, l in d.items()}


def medir_examen(model_path, personal_file, lineas, congelada):
    """medir.py sobre el examen: el dict de resultados de `medir.medir` (M0)."""
    from types import SimpleNamespace

    import medir
    from onepepe_math.classifier import SymbolClassifier
    from onepepe_math.personal import PersonalModel
    from onepepe_math.segment import SegmentParams

    clf = medir.Cache(SymbolClassifier(model_path))
    with tempfile.TemporaryDirectory() as d:
        ruta = os.path.join(d, "base.jsonl")
        shutil.copy2(personal_file, ruta)  # el kNN es el de siempre; el original no se toca
        pm = PersonalModel(clf, ruta, background=False)
        args = SimpleNamespace(incremental=None, ref="O", sin_variantes=False)
        return medir.medir(lineas, lambda _l: (clf, pm), args, SegmentParams.load(), congelada)


def _validacion(personal_file):
    """(base, validación) de tus ejemplos, sin examen, para cuando no hay examen_trazos.json."""
    from train_classifier import reparto_propio

    ruta = os.path.join(MATH, "examen.json")
    conj = json.load(open(ruta, encoding="utf-8")) if os.path.exists(ruta) else {}
    tr, va = reparto_propio(load_examples(personal_file), conj)
    return [(e, s) for e, s, _ in tr if e != "__trozo__"], [(e, s) for e, s, _ in va if e != "__trozo__"]


def aceptar(old_path, new_path, personal_file):
    """M2.6: (cambiar?, mensaje). Se cambia solo si (1) el examen (corte y etiqueta) no empeora (P > 0,20), (2) el
    top-1 de MathWriting reservado no baja más de 2 puntos y (3) las sustituciones erróneas no suben. Sin
    examen_trazos.json, la validación hace de examen en (1) y (3)."""
    from onepepe_math.classifier import SymbolClassifier
    from train_classifier import top1_reservado

    t_old = top1_reservado(SymbolClassifier(old_path), DATASET)[0]
    t_new = top1_reservado(SymbolClassifier(new_path), DATASET)[0]
    datos = _lineas_examen()
    if datos:
        import medir

        res = [medir_examen(p, personal_file, *datos) for p in (old_path, new_path)]
        with tempfile.TemporaryDirectory() as d:
            rutas = []
            for i, r in enumerate(res):
                rutas.append(os.path.join(d, f"{i}.json"))
                json.dump(r, open(rutas[-1], "w", encoding="utf-8"))
            p = medir.comparar(*rutas)["P"]
        c1 = p > 0.20
        m = [r["metricas"] for r in res]
        e_old, e_new = m[0]["sust_erroneas"]["k"], m[1]["sust_erroneas"]["k"]
        texto = (f"Examen: corte y etiqueta {m[0]['corte_y_etiqueta']['k']} -> {m[1]['corte_y_etiqueta']['k']} "
                 f"(P {p:.2f}), sustituciones erróneas {e_old} -> {e_new}")
    else:
        from evaluar_mi_letra import score
        from onepepe_math.personal import PersonalModel

        base, test = _validacion(personal_file)
        s = []
        for path in (old_path, new_path):
            clf = SymbolClassifier(path)
            with tempfile.TemporaryDirectory() as d:
                pm = PersonalModel(clf, os.path.join(d, "base.jsonl"), background=False)
                for e, st in base:
                    pm.add_correction(e, st)
                s.append(score(clf, test, pm))
        c1 = s[1][0] >= s[0][0]
        e_old, e_new = s[0][1] - s[0][2], s[1][1] - s[1][2]
        texto = (f"Validación ({len(test)} símbolos): aciertos {s[0][0]} -> {s[1][0]}, "
                 f"sustituciones erróneas {e_old} -> {e_new}")
    ok = c1 and t_new >= t_old - 0.02 and e_new <= e_old
    return ok, (f"{texto}; MathWriting reservado {t_old:.1%} -> {t_new:.1%}. "
                + ("Se usa el modelo nuevo." if ok else "El nuevo no cumple las tres condiciones: se mantiene el anterior."))


def retrain():
    start = time.time()
    if not os.path.isdir(DATASET):
        emit(stage="error", message="Falta MathWriting: ejecuta una vez tools/onepepe-math/instalar.sh --entrenar")
        return 1
    items = load_examples()
    emit(stage="start", progress=0.02, message=f"{len(items)} ejemplos tuyos")

    with tempfile.TemporaryDirectory() as d:
        personal_file = os.path.join(d, "personal.jsonl")
        with open(personal_file, "w", encoding="utf-8") as f:
            for it in items:  # todo: el entrenamiento aparta solo el examen y la validación (examen.json)
                f.write(json.dumps(it, ensure_ascii=False) + "\n")
        new_model = os.path.join(d, "simbolos.npz")
        cmd = [sys.executable, os.path.join(DIR, "train_classifier.py"), DATASET, "-o", new_model,
               "--personal", personal_file, "--personal-weight", str(int(load_last().get("personal_weight", 30)))]
        stages = [("símbolos cargados", 0.15, "Datos cargados"), ("ejemplos de 'trozo'", 0.25, "Preparando ejemplos"),
                  ("entrenando con", 0.3, "Entrenando la red"), ("entrenado en", 0.8, "Red entrenada"),
                  ("modelo de contexto", 0.9, "Modelo de contexto listo")]
        proc = subprocess.Popen(
            cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, cwd=DIR, preexec_fn=_die_with_parent
        )

        def cancel(_signum, _frame):
            # «Cancelar» en la app: para el entrenamiento y sale sin tocar el modelo
            proc.terminate()
            emit(stage="cancelled", message="Cancelado; se mantiene el modelo actual")
            sys.exit(130)

        signal.signal(signal.SIGTERM, cancel)
        signal.signal(signal.SIGINT, cancel)
        for line in proc.stdout:
            for key, progress, message in stages:
                if key in line:
                    emit(stage="train", progress=progress, message=message)
        if proc.wait() != 0 or not os.path.exists(new_model):
            emit(stage="error", message="El entrenamiento falló; se mantiene el modelo actual")
            return 1

        replaced = True
        message = "Modelo nuevo listo"
        if os.path.exists(MODEL):
            emit(stage="compare", progress=0.95, message="Comparando con el modelo actual")
            replaced, message = aceptar(MODEL, new_model, personal_file)
        import numpy as np

        extra = np.load(new_model)
        hecho = {"personal_weight": int(extra["personal_weight"]), "epocas": int(extra["epocas"])}
        if replaced:
            os.makedirs(MATH, exist_ok=True)
            if os.path.exists(MODEL):
                shutil.copy2(MODEL, PREVIOUS)
            shutil.copy2(new_model, MODEL)

    with open(LAST, "w", encoding="utf-8") as f:
        json.dump({"seconds": round(time.time() - start), "date": time.strftime("%Y-%m-%dT%H:%M:%S"),
                   "examples": len(items),
                   "symbols": sum(1 for it in items if it.get("kind", "symbol") != "junk"),
                   "replaced": replaced, **hecho}, f)
    emit(stage="done", progress=1.0, replaced=replaced, message=message)
    return 0


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if "--estimar" in argv:
        emit(**estimate())
        return 0
    return retrain()


if __name__ == "__main__":
    sys.exit(main())
