# SPDX-License-Identifier: GPL-3.0-or-later
"""Tu letra: correcciones guardadas en tu equipo y clasificador personal por vecinos más cercanos.

Cada corrección (los trazos del símbolo y la etiqueta correcta) se añade a
~/.local/share/onepepe/math/correcciones.jsonl. Nunca se envía a ningún sitio.
"""

import json
import os

import numpy as np

DEFAULT_PATH = os.path.join(
    os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math", "correcciones.jsonl"
)


class PersonalModel:
    def __init__(self, classifier, path=DEFAULT_PATH):
        self.classifier = classifier
        self.path = path
        self.labels = []
        self.embeddings = []
        if os.path.exists(path):
            with open(path, encoding="utf-8") as f:
                for line in f:
                    try:
                        item = json.loads(line)
                        self._add(item["latex"], item["strokes"])
                    except (ValueError, KeyError):
                        continue

    def _add(self, latex, strokes):
        e = self.classifier.embedding(strokes)
        self.labels.append(latex)
        self.embeddings.append(e / (np.linalg.norm(e) + 1e-9))

    def add_correction(self, latex, strokes):
        """Guarda un ejemplo de tu letra."""
        self._add(latex, strokes)
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        with open(self.path, "a", encoding="utf-8") as f:
            f.write(json.dumps({"latex": latex, "strokes": strokes}) + "\n")

    def __len__(self):
        return len(self.labels)

    def adjust(self, strokes, cands, k=5, min_similarity=0.85):
        """Mezcla las candidatas del modelo general con los vecinos más parecidos de tu letra.

        Cuantos más ejemplos tuyos muy parecidos hay, más pesan.
        """
        if not self.labels:
            return cands
        e = self.classifier.embedding(strokes)
        e = e / (np.linalg.norm(e) + 1e-9)
        sims = np.array(self.embeddings) @ e
        order = np.argsort(-sims)[:k]
        votes = {}
        for i in order:
            if sims[i] >= min_similarity:
                votes[self.labels[i]] = votes.get(self.labels[i], 0.0) + float(sims[i])
        if not votes:
            return cands
        n_close = sum(1 for i in order if sims[i] >= min_similarity)
        weight = n_close / (n_close + 1.0)
        z = sum(votes.values())
        mixed = {lab: (1 - weight) * p for lab, p in cands}
        for lab, v in votes.items():
            mixed[lab] = mixed.get(lab, 0.0) + weight * v / z
        zz = sum(mixed.values())
        return sorted(((lab, p / zz) for lab, p in mixed.items()), key=lambda x: -x[1])
