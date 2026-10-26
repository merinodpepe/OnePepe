# SPDX-License-Identifier: GPL-3.0-or-later
"""Reescoring de los símbolos con contexto.

1. Vecinos: un modelo de pares de símbolos (bigramas) aprendido de fórmulas reales decide entre lecturas con la misma
   forma (× o x entre cifras, 0 u o, etc.). Se combina con el clasificador mediante Viterbi.
2. Glosario: los símbolos habituales del cuaderno parten con ventaja.
3. Grupos por forma: los símbolos que se parecen mucho entre sí (tu forma de escribir la K) comparten etiqueta; los
   dudosos heredan la de los claros.
"""

import math

import numpy as np

BOS = "<s>"


class BigramModel:
    def __init__(self, vocab, unigram, bigram):
        self.vocab = list(vocab)
        self.index = {v: i for i, v in enumerate(self.vocab)}
        self.unigram = unigram
        self.bigram = bigram

    @classmethod
    def train(cls, sequences, vocab, k=0.1):
        vocab = [BOS] + [v for v in vocab if v != BOS]
        index = {v: i for i, v in enumerate(vocab)}
        n = len(vocab)
        uni = np.full(n, k)
        bi = np.full((n, n), k)
        for seq in sequences:
            prev = 0
            for tok in seq:
                i = index.get(tok)
                if i is None:
                    prev = 0
                    continue
                uni[i] += 1
                bi[prev, i] += 1
                prev = i
        uni = uni / uni.sum()
        bi = bi / bi.sum(axis=1, keepdims=True)
        return cls(vocab, uni.astype(np.float32), bi.astype(np.float32))

    def log_unigram(self, tok):
        i = self.index.get(tok)
        return math.log(self.unigram[i]) if i is not None else math.log(1e-6)

    def log_bigram(self, prev, tok):
        i, j = self.index.get(prev, 0), self.index.get(tok)
        if j is None:
            return math.log(1e-6)
        return math.log(self.bigram[i, j])


def reading_order(symbols, boxes):
    """Índices de los símbolos de izquierda a derecha (por el centro de su caja)."""
    return sorted(range(len(symbols)), key=lambda i: (boxes[i][0] + boxes[i][2]) / 2)


def viterbi(cand_lists, lm, lm_weight=1.0, prior_weight=0.3):
    """Mejor secuencia de etiquetas dadas las candidatas de cada símbolo (en orden de lectura).

    Devuelve, para cada símbolo, las candidatas reordenadas con probabilidades a posteriori aproximadas.
    """
    n = len(cand_lists)
    if n == 0:
        return []
    # forward
    scores = []
    back = []
    for t, cands in enumerate(cand_lists):
        cur = []
        bp = []
        for lab, p in cands:
            emit = math.log(max(p, 1e-9)) + prior_weight * lm.log_unigram(lab)
            if t == 0:
                cur.append(emit + lm_weight * lm.log_bigram(BOS, lab))
                bp.append(-1)
            else:
                best, arg = -math.inf, -1
                for k, (plab, _) in enumerate(cand_lists[t - 1]):
                    s = scores[t - 1][k] + lm_weight * lm.log_bigram(plab, lab)
                    if s > best:
                        best, arg = s, k
                cur.append(best + emit)
                bp.append(arg)
        scores.append(cur)
        back.append(bp)
    # backward pass for the best path
    path = [int(np.argmax(scores[-1]))]
    for t in range(n - 1, 0, -1):
        path.append(back[t][path[-1]])
    path.reverse()
    # posterior-like rescoring: emission x context from the chosen neighbours
    out = []
    for t, cands in enumerate(cand_lists):
        prev = cand_lists[t - 1][path[t - 1]][0] if t > 0 else BOS
        nxt = cand_lists[t + 1][path[t + 1]][0] if t + 1 < n else None
        logs = []
        for lab, p in cands:
            s = math.log(max(p, 1e-9)) + prior_weight * lm.log_unigram(lab) + lm_weight * lm.log_bigram(prev, lab)
            if nxt is not None:
                s += lm_weight * lm.log_bigram(lab, nxt)
            logs.append(s)
        m = max(logs)
        e = [math.exp(v - m) for v in logs]
        z = sum(e)
        ranked = sorted(zip([c for c, _ in cands], [v / z for v in e]), key=lambda x: -x[1])
        out.append(ranked)
    return out


def apply_glossary(cands, glossary, boost=2.0):
    """Multiplica la probabilidad de los símbolos del glosario y renormaliza."""
    if not glossary:
        return cands
    w = [(lab, p * (boost if lab in glossary else 1.0)) for lab, p in cands]
    z = sum(p for _, p in w) or 1.0
    return sorted(((lab, p / z) for lab, p in w), key=lambda x: -x[1])


def shape_groups(embeddings, threshold=0.85):
    """Agrupa símbolos con forma casi igual (similitud del coseno de sus representaciones internas)."""
    n = len(embeddings)
    if n == 0:
        return []
    e = np.array(embeddings, dtype=np.float32)
    e = e / (np.linalg.norm(e, axis=1, keepdims=True) + 1e-9)
    sim = e @ e.T
    group = list(range(n))

    def find(i):
        while group[i] != i:
            group[i] = group[group[i]]
            i = group[i]
        return i

    for i in range(n):
        for j in range(i + 1, n):
            if sim[i, j] >= threshold:
                group[find(i)] = find(j)
    return [find(i) for i in range(n)]


def share_within_groups(cand_lists, groups, strength=0.7):
    """Los símbolos de un mismo grupo mezclan sus probabilidades; los seguros pesan más que los dudosos."""
    members = {}
    for i, g in enumerate(groups):
        members.setdefault(g, []).append(i)
    out = list(cand_lists)
    for idx in members.values():
        if len(idx) < 2:
            continue
        pooled = {}
        for i in idx:
            conf = cand_lists[i][0][1] if cand_lists[i] else 0.0
            for lab, p in cand_lists[i]:
                pooled[lab] = pooled.get(lab, 0.0) + conf * p
        z = sum(pooled.values()) or 1.0
        for i in idx:
            mixed = {lab: (1 - strength) * p for lab, p in cand_lists[i]}
            for lab, p in pooled.items():
                mixed[lab] = mixed.get(lab, 0.0) + strength * p / z
            zz = sum(mixed.values()) or 1.0
            out[i] = sorted(((lab, p / zz) for lab, p in mixed.items()), key=lambda x: -x[1])
    return out
