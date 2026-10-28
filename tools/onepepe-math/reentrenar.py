#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""OnePepe: reentrenar el reconocedor de símbolos con tu letra (lo usa el botón «Reentrenar» de la app).

  --estimar   dice cuántos ejemplos tuyos hay y cuánto tardará (según lo que tardó la vez anterior), en una línea JSON.
  (nada)      reentrena: escribe el progreso en líneas JSON {"stage", "progress", "message"} y al final
              {"stage": "done", "replaced": bool, "message"}.

Cómo reentrena:
  1. Los ejemplos tuyos más antiguos (70 %) entran en el entrenamiento, junto con MathWriting. Los más recientes (30 %)
     se reservan para comparar; siguen en el kNN mientras usas la app y entrarán en el próximo reentrenamiento.
  2. Compara el modelo nuevo con el actual en esos ejemplos reservados (red + kNN). Solo lo cambia si no empeora;
     el anterior se guarda como simbolos.anterior.npz.
  3. Apunta cuánto tardó, para estimar la próxima vez.
Con menos de 10 ejemplos tuyos no hay con qué comparar: entra todo y el modelo se cambia.
Todo ocurre en tu equipo.
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

MIN_TO_COMPARE = 10
HOLDOUT = 0.3
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


def compare(old_path, new_path, base, test):
    """Acierto (red + kNN) de cada modelo en los ejemplos reservados: (viejo, nuevo)."""
    from evaluar_mi_letra import score
    from onepepe_math.classifier import SymbolClassifier
    from onepepe_math.personal import PersonalModel

    results = []
    test_symbols = [(it["latex"], it["strokes"]) for it in test if it.get("kind", "symbol") != "junk"]
    for path in (old_path, new_path):
        clf = SymbolClassifier(path)
        with tempfile.TemporaryDirectory() as d:
            personal = PersonalModel(clf, os.path.join(d, "base.jsonl"), background=False)
            for it in base:
                if it.get("kind", "symbol") != "junk":
                    personal.add_correction(it["latex"], it["strokes"])
            ok, _sure, sure_ok = score(clf, test_symbols, personal)
            results.append((ok, sure_ok))
    return results, len(test_symbols)


def retrain():
    start = time.time()
    if not os.path.isdir(DATASET):
        emit(stage="error", message="Falta MathWriting: ejecuta una vez tools/onepepe-math/instalar.sh --entrenar")
        return 1
    items = load_examples()
    compare_possible = os.path.exists(MODEL) and len(items) >= MIN_TO_COMPARE
    n_base = int(len(items) * (1 - HOLDOUT)) if compare_possible else len(items)
    base, test = items[:n_base], items[n_base:]
    emit(stage="start", progress=0.02, message=f"{len(items)} ejemplos tuyos ({len(base)} para entrenar)")

    with tempfile.TemporaryDirectory() as d:
        personal_file = os.path.join(d, "personal.jsonl")
        with open(personal_file, "w", encoding="utf-8") as f:
            for it in base:
                f.write(json.dumps(it, ensure_ascii=False) + "\n")
        new_model = os.path.join(d, "simbolos.npz")
        cmd = [sys.executable, os.path.join(DIR, "train_classifier.py"), DATASET, "-o", new_model,
               "--personal", personal_file]
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
        if compare_possible:
            emit(stage="compare", progress=0.95, message="Comparando con el modelo actual")
            ((old_ok, old_sure), (new_ok, new_sure)), n = compare(MODEL, new_model, base, test)
            replaced = (new_ok, new_sure) >= (old_ok, old_sure)
            message = (
                f"Con tus ejemplos recientes: antes {old_ok}/{n}, ahora {new_ok}/{n}. "
                + ("Se usa el modelo nuevo." if replaced else "El nuevo no mejora: se mantiene el anterior.")
            )
        if replaced:
            os.makedirs(MATH, exist_ok=True)
            if os.path.exists(MODEL):
                shutil.copy2(MODEL, PREVIOUS)
            shutil.copy2(new_model, MODEL)

    with open(LAST, "w", encoding="utf-8") as f:
        json.dump({"seconds": round(time.time() - start), "date": time.strftime("%Y-%m-%dT%H:%M:%S"),
                   "examples": len(items),
                   "symbols": sum(1 for it in items if it.get("kind", "symbol") != "junk"),
                   "replaced": replaced}, f)
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
