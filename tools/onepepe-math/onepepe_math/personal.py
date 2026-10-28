# SPDX-License-Identifier: GPL-3.0-or-later
"""Tu letra: almacén local de correcciones y clasificador por vecinos más cercanos (kNN) sobre el embedding de la red.

Cubre el hueco entre reentrenamientos: cada corrección cuenta al instante, sin reentrenar. Al reentrenar
(`train_classifier.py --personal`), estos mismos ejemplos entran en la red.

Cada ejemplo se guarda en ~/.local/share/onepepe/math/correcciones.jsonl (nunca sale de tu equipo) con:
id, trazos (crudos), etiqueta correcta, etiqueta predicha, embedding, versión del modelo, fecha y cuaderno.
Si la versión del modelo cambia (tras reentrenar), los embeddings se recalculan en segundo plano desde los trazos.

Mezcla: k vecinos, peso exp(-d/τ) con d = 1 - coseno → p_knn. λ = λ_max · cercanía · coherencia, y λ = 0 si el
vecino más cercano está a más de d_max. p_final = (1-λ)·p_red + λ·p_knn, renormalizada. Con el almacén vacío, el
resultado es idéntico al de la red sola.
"""

import json
import math
import os
import threading
import time
import uuid

import numpy as np

DEFAULT_PATH = os.path.join(
    os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math", "correcciones.jsonl"
)

# Medido con tu letra (30 expresiones, división temporal): ver MATEMATICAS-INFORME.md, sección 0.5.
K = 5
TAU = 0.1
D_MAX = 0.3
LAMBDA_MAX = 0.6


def _unit(e):
    e = np.asarray(e, dtype=np.float32)
    return e / (np.linalg.norm(e) + 1e-9)


class NeighborIndex:
    """Vecinos más cercanos por fuerza bruta en memoria. La interfaz (add, remove, neighbors) permite cambiarlo por
    un índice aproximado (HNSW) si algún día hay muchos miles de ejemplos."""

    def __init__(self):
        self.ids = []
        self.matrix = np.zeros((0, 0), dtype=np.float32)

    def __len__(self):
        return len(self.ids)

    def rebuild(self, items):
        """`items`: lista de (id, embedding unitario)."""
        self.ids = [i for i, _ in items]
        self.matrix = np.array([e for _, e in items], dtype=np.float32) if items else np.zeros((0, 0), np.float32)

    def neighbors(self, embedding, k):
        """Los k más cercanos: lista de (distancia = 1 - coseno, id), de menor a mayor distancia."""
        if not self.ids:
            return []
        d = 1.0 - self.matrix @ _unit(embedding)
        order = np.argsort(d)[:k]
        return [(float(d[i]), self.ids[i]) for i in order]


class PersonalModel:
    def __init__(self, classifier, path=DEFAULT_PATH, background=True):
        self.classifier = classifier
        self.path = path
        self.version = getattr(classifier, "version", "")
        self.examples = {}  # id -> registro
        self.index = NeighborIndex()
        self._lock = threading.Lock()
        stale = []
        if os.path.exists(path):
            with open(path, encoding="utf-8") as f:
                for line in f:
                    try:
                        item = json.loads(line)
                        item.setdefault("id", uuid.uuid4().hex[:12])
                        item["strokes"] = [[tuple(p) for p in s] for s in item["strokes"]]
                        item["latex"]
                    except (ValueError, KeyError, TypeError):
                        continue
                    self.examples[item["id"]] = item
                    if item.get("model") != self.version or "embedding" not in item:
                        stale.append(item["id"])
        self._rebuild_index()
        if stale:
            if background:
                threading.Thread(target=self._refresh, args=(stale,), daemon=True).start()
            else:
                self._refresh(stale)

    # ----- almacén

    def _refresh(self, ids):
        """Recalcula embeddings de ejemplos guardados con otra versión del modelo (o sin embedding)."""
        for i in ids:
            item = self.examples.get(i)
            if item is None:
                continue
            item["embedding"] = [round(float(v), 5) for v in _unit(self.classifier.embedding(item["strokes"]))]
            item["model"] = self.version
        with self._lock:
            self._rebuild_index()
            self._save_all()

    def _rebuild_index(self):
        self.index.rebuild(
            [
                (i, _unit(item["embedding"]))
                for i, item in self.examples.items()
                if item.get("model") == self.version and "embedding" in item and item.get("kind") != "junk"
            ]
        )

    def _save_all(self):
        os.makedirs(os.path.dirname(self.path) or ".", exist_ok=True)
        tmp = self.path + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            for item in self.examples.values():
                f.write(json.dumps(item, ensure_ascii=False) + "\n")
        os.replace(tmp, self.path)

    def __len__(self):
        return len(self.index)

    def count(self, kind=None):
        """Ejemplos guardados (de un tipo, o todos)."""
        return sum(1 for item in self.examples.values() if kind is None or item.get("kind", "symbol") == kind)

    def add_correction(self, latex, strokes, predicted=None, notebook=None, embedding=None, kind="symbol"):
        """Guarda un ejemplo de tu letra (una corrección, o un símbolo que aceptas tal cual). Devuelve su id.

        `kind="junk"`: un "trozo" (medio símbolo, o dos juntos) para enseñar a cortar; no entra en el kNN, solo en
        el reentrenamiento."""
        if embedding is None:
            embedding = self.classifier.embedding(strokes)
        item = {
            "id": uuid.uuid4().hex[:12],
            "latex": latex,
            "predicted": predicted,
            "strokes": [[(round(float(x), 2), round(float(y), 2)) for x, y in s] for s in strokes],
            "embedding": [round(float(v), 5) for v in _unit(embedding)],
            "model": self.version,
            "date": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "notebook": notebook,
            "kind": kind,
        }
        with self._lock:
            self.examples[item["id"]] = item
            self._rebuild_index()
            os.makedirs(os.path.dirname(self.path) or ".", exist_ok=True)
            with open(self.path, "a", encoding="utf-8") as f:
                f.write(json.dumps(item, ensure_ascii=False) + "\n")
        return item["id"]

    def list_examples(self):
        """Los ejemplos guardados, sin trazos ni embedding (para mostrarlos y poder borrarlos o cambiarlos)."""
        keys = ("id", "latex", "predicted", "date", "notebook", "kind")
        return [{k: item.get(k) for k in keys} for item in self.examples.values()]

    def forget(self, example_id):
        """Borra un ejemplo (p. ej. una corrección equivocada)."""
        with self._lock:
            if self.examples.pop(example_id, None) is None:
                return False
            self._rebuild_index()
            self._save_all()
        return True

    def relabel(self, example_id, latex):
        """Cambia la etiqueta correcta de un ejemplo."""
        with self._lock:
            item = self.examples.get(example_id)
            if item is None:
                return False
            item["latex"] = latex
            self._save_all()
        return True

    # ----- vecinos y mezcla

    def neighbors(self, embedding, k=K):
        """(distancia, etiqueta, id) de los k ejemplos tuyos más parecidos."""
        return [(d, self.examples[i]["latex"], i) for d, i in self.index.neighbors(embedding, k) if i in self.examples]

    def adjust(self, strokes, cands, embedding=None):
        """Mezcla las candidatas de la red con los vecinos de tu letra. Sin vecinos cercanos, no cambia nada."""
        if not len(self.index):
            return cands
        if embedding is None:
            embedding = self.classifier.embedding(strokes)
        near = [(d, lab) for d, lab, _ in self.neighbors(embedding, K) if d <= D_MAX]
        if not near:
            return cands
        votes = {}
        for d, lab in near:
            votes[lab] = votes.get(lab, 0.0) + math.exp(-d / TAU)
        z = sum(votes.values())
        p_knn = {lab: v / z for lab, v in votes.items()}
        closeness = math.exp(-near[0][0] / TAU)
        coherence = max(p_knn.values())
        lam = LAMBDA_MAX * closeness * coherence
        mixed = {lab: (1.0 - lam) * p for lab, p in cands}
        for lab, p in p_knn.items():
            mixed[lab] = mixed.get(lab, 0.0) + lam * p
        zz = sum(mixed.values())
        return sorted(((lab, p / zz) for lab, p in mixed.items()), key=lambda x: -x[1])
