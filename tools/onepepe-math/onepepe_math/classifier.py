# SPDX-License-Identifier: GPL-3.0-or-later
"""Clasificador de símbolos: red neuronal pequeña evaluada solo con numpy.

El modelo se guarda en un archivo .npz (pesos, nombres de clases y normalización) que se genera con
`train_classifier.py` y no se sube al repositorio.
"""

import hashlib
import os

import numpy as np

from .features import symbol_features

DEFAULT_MODEL = os.path.join(
    os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math", "simbolos.npz"
)


# Clase especial: grupo de trazos que no es un símbolo completo.
JUNK = "__trozo__"


class SymbolClassifier:
    def __init__(self, path=DEFAULT_MODEL, vocabulary=None):
        data = np.load(path, allow_pickle=False)
        # versión del modelo: cambia en cada reentrenamiento (los embeddings guardados se recalculan)
        if "version" in data:
            self.version = str(data["version"])
        else:
            with open(path, "rb") as f:
                self.version = hashlib.sha1(f.read()).hexdigest()[:12]
        self.classes = [str(c) for c in data["classes"]]
        self.mean = data["mean"]
        self.std = data["std"]
        n = int(data["n_layers"])
        self.weights = [(data[f"W{i}"], data[f"b{i}"]) for i in range(n)]
        # clases permitidas (las demás se ignoran); por defecto, el vocabulario básico
        if vocabulary is None:
            from .symbols import CORE

            vocabulary = CORE
        from .symbols import normalize_label

        allowed = set(vocabulary) | {JUNK}
        # una clase con otro nombre para el mismo símbolo (\longrightarrow = \rightarrow) cuenta como él
        self.labels = [normalize_label(c) if c != JUNK else c for c in self.classes]
        self.mask = np.array([label in allowed for label in self.labels])
        self.lm = None
        if "lm_vocab" in data:
            from .context import BigramModel

            self.lm = BigramModel([str(v) for v in data["lm_vocab"]], data["lm_unigram"], data["lm_bigram"])

    def embedding(self, strokes):
        """Representación interna (última capa oculta), para comparar formas entre símbolos."""
        x = (symbol_features(strokes) - self.mean) / self.std
        for w, b in self.weights[:-1]:
            x = np.maximum(x @ w + b, 0.0)
        return x

    def probabilities_and_embedding(self, strokes):
        """(probabilidades, embedding) en una sola pasada por la red."""
        x = (symbol_features(strokes) - self.mean) / self.std
        embedding = None
        for i, (w, b) in enumerate(self.weights):
            x = x @ w + b
            if i < len(self.weights) - 1:
                x = np.maximum(x, 0.0)
                embedding = x
        x = np.where(self.mask, x, -np.inf)
        x = x - x.max()
        e = np.exp(x)
        return e / e.sum(), embedding

    def probabilities(self, strokes):
        x = (symbol_features(strokes) - self.mean) / self.std
        for i, (w, b) in enumerate(self.weights):
            x = x @ w + b
            if i < len(self.weights) - 1:
                x = np.maximum(x, 0.0)
        x = np.where(self.mask, x, -np.inf)
        x = x - x.max()
        e = np.exp(x)
        return e / e.sum()

    def candidates(self, strokes, k=5):
        """Las k etiquetas más probables: lista de (latex, probabilidad), sin la clase "trozo"."""
        return self.candidates_w_junk(strokes, k)[0]

    def candidates_w_junk(self, strokes, k=5):
        """(candidatas sin "trozo", probabilidad de que el grupo sea un trozo y no un símbolo)."""
        p = self.probabilities(strokes)
        junk = float(p[self.classes.index(JUNK)]) if JUNK in self.classes else 0.0
        # las clases que son el mismo símbolo suman su probabilidad
        merged = {}
        for i in np.flatnonzero(self.mask):
            if self.classes[i] != JUNK:
                merged[self.labels[i]] = merged.get(self.labels[i], 0.0) + float(p[i])
        best = sorted(merged.items(), key=lambda x: -x[1])[:k]
        return best, junk
