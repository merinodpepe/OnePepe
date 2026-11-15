# Informe del cierre (2026-11-15)

## En cinco líneas

- Hechas y fusionadas 11 de las 14 tareas del plan (T0–T8, T10, T11 y el código de T13). T9 saltada (faltan páginas
  reales); T12 (diseño) es el último paso.
- Los resultados que se enseñaban mal ya no salen: las sondas pasan de 8 resultados erróneos a 0 (de 24).
- El reconocimiento mejora pero **no llega a las metas**: símbolos con corte y etiqueta, de 44–66 % a 78,3 % en el
  examen (meta 90 %).
- El freno principal es el corte: símbolos juntados (13,4 % en letra no vista) y rayas de fracción.
- Nada está probado con lápiz. Tus datos reales no se han tocado: instalar el modelo nuevo es una orden tuya.

## Tareas

| Tarea | Estado | Rama | Versión | Aceptación |
|-------|--------|------|---------|------------|
| T0 Banco de medida | hecha | `feat/banco-de-medida` | 1.5.0 | Cumplida (línea base 91,4 / 80,2 frente a 90,0 / 79,0 de la auditoría, por la penalización LEJOS) |
| T1 Reimportar entrenos | hecha | `fix/reimportar-entrenos` | 1.6.0 | Cumplida |
| T2 Entrenamiento sin fugas | hecha | `fix/entrenamiento-sin-fugas` | 1.6.0 | Empate en corte y etiqueta; sustituciones erróneas 23 → 9 |
| T3 Evaluación correcta | hecha | `fix/evaluacion-correcta` | 1.6.0 | Cumplida: sondas erróneas 8 → 0 |
| T4 Entorno | hecha | `feat/entorno` | 1.6.0 | Cumplida |
| T11 Captura de fallos | hecha (sin pantalla de ejemplos) | `feat/captura-fallos` | 1.6.0 | Cumplida; interfaz sin abrir |
| T5 Corte | hecha; «rayas primero» apagado | `feat/corte-rayas` | 1.7.0 | No cumplida: raya 25 % (meta 85), juntado 12,4 % (meta 4) |
| T6 Léxico | hecha | `feat/lexico` | 1.7.0 | No cumplida: números 59,1 % (meta 90) en ese momento |
| T7a Extracción | hecha (una ronda) | `feat/extraer-simbolos` | 1.8.0 | 212 541 símbolos |
| T8 Confianza | hecha | `feat/confianza` | 1.8.0 | No cumplida: ningún umbral baja del 2 % de error |
| T7b Clasificador v2 | hecha, modelo v2 en la copia de trabajo | `feat/clasificador-v2` | 1.8.0 | No cumplida: top-1 65,9–72,8 % (meta 80) |
| T10 N mejores | hecha; búsqueda de lecturas apagada | `feat/n-mejores` | 1.9.0 | No cumplida: +1 renglón (empate) |
| T13 Cierre | código hecho; promoción sin aplicar | `feat/cierre` | 1.9.0 | Metas medidas, no cumplidas |
| T9 Renglón vivo | saltada | — | — | Faltan páginas reales de apuntes |
| T5b Modelo de pares | no abierta | — | — | Es lo siguiente si se sigue |
| T12 Diseño fluido | último paso | `feat/diseno-fluido` | — | Ver `ESTADO.md` |

## Cifras contra las metas

Medida final con el modelo v2. Entre corchetes, intervalo al 90 %. «Incremental» simula escribir con pausas de 0,8 s.

| Medida | Hoy (auditoría) | Final examen (n = 217; 31 renglones) | Final cruzada (n = 813; 108 renglones) | Examen incremental | Meta | ¿Cumplida? |
|--------|-----------------|------|------|------|------|------|
| Símbolos con corte y etiqueta | 44–66 % | 78,3 % [73,4–82,6] | 70,2 % [67,5–72,8] | 76,0 % | ≥ 90 % | No |
| Renglones todos bien | 5–28 % | 54,8 % [40,3–68,6] | 38,0 % [30,7–45,9] | 45,2 % | ≥ 60 % | No |
| Números de 2 o más cifras | 63–78 % | 85,7 % (6 de 7) | 77,3 % (17 de 22) | 85,7 % | ≥ 90 % | No (muestra pequeña) |
| Raya de fracción, corte y papel | 21 % | 16,7 % (1 de 6) | 10,7 % (3 de 28) | 16,7 % | ≥ 90 % | No |
| Sustituciones erróneas (sin umbral) | 9–10,5 % | 12,6 % (21 de 167) | 19,7 % (127 de 644) | 16,2 % | ≤ 2 % | No |
| Resultados erróneos en las sondas | 8 | 0 de 24 | — | — | 0 | **Sí** |

Otras: corte exacto 89,9 % (examen) y 83,9 % (cruzada); juntado 6,9 % y 13,4 %; red sola top-1 72,8 % y 65,9 %,
top-5 92,6 %; 13–15 ms por renglón (meta < 30: cumplida). Con el umbral calibrado (0,999) solo se sustituye el
3–5 % de los símbolos.

Aviso sobre «sustituciones erróneas»: la cifra final se mide sustituyéndolo todo (umbral 0) y la de la auditoría,
con el umbral 0,95. No son comparables directamente.

## Decisiones tomadas sin preguntar

Todas están en [`DECISIONES.md`](DECISIONES.md), secciones «Cierre, oleada…». Las que conviene que revises:

1. **No he instalado nada en tus datos reales**, aunque dijiste que sí (A1): las metas no se cumplen y hay una
   elección tuya pendiente (punto 2).
2. **Umbral de confianza.** El calibrado es 0,999: casi no sustituye símbolos. El de la app es 0,95: sustituye
   alrededor del 30 % y falla 1 de cada 10.
3. **«=» y «Variables» quedan encendidos por defecto** (A4), porque las sondas dan 0 erróneos.
4. **La parada del entrenamiento se queda como está** (2 épocas): más épocas empeoran tu letra.
5. **Constantes del kNN:** se descartó un ajuste hecho sobre el examen; quedan las anteriores.
6. **Apagado por «si no mejora, no se queda»:** «rayas primero» (T5) y la búsqueda de lecturas alternativas (T10).
7. **No hechas:** segunda ronda de extracción, curva de aprendizaje, red convolucional (A6), pantalla de ejemplos.
8. **Cambios de comportamiento:** un hueco de más de 0,8 alturas entre cifras da «?»; `e` es la constante salvo que
   la definas; un ciclo de definiciones da «?».
9. **`docs/archivo/` está fuera de git** (venía del punto de guardado que no se sube).

## Qué está comprobado y cómo

| Parte | Compilado | Tests | Ejecutado con tus datos (copia de trabajo) | Probado con lápiz |
|-------|-----------|-------|---------------------------------------------|-------------------|
| Programa auxiliar (Python) | — | 216 pasan | Sí: medida en examen, cruzada e incremental | No |
| Motor (Rust) | Sí | 27 pasan (filtro `math`) | — | No |
| Interfaz (botón «Esto está mal», resultado dudoso) | Solo `cargo check` | — | No se ha abierto la app | No |
| Traducción | `msgfmt -c` ok | — | — | No |
| `promover_cierre.py` | — | 5 pasan | Solo modo informe: no escribió nada | — |

Rust se comprueba con `~/.local/share/onepepe/cierre-trabajo/rust.sh engine|ui|po` (SDK de Flatpak).

## Lo que queda para ti

**1. Instalar la versión nueva de la app** (compila 20–40 min):

```bash
cd /home/pepe/Pepe_Flock/One_Pepe && git checkout onepepe && git pull --ff-only
cd tools/onepepe-math && ./instalar.sh && cd ../..
~/instalar-onepepe.sh
```

**2. Instalar el modelo y el almacén nuevos** (hace copia `.bak` de cada archivo). Elige una:

```bash
cd /home/pepe/Pepe_Flock/One_Pepe/tools/onepepe-math
# a) con el umbral de la app (0,95): sigue sustituyendo símbolos, falla alrededor de 1 de cada 10
ONEPEPE_DATA=~/.local/share/onepepe/cierre-trabajo ~/.local/share/onepepe/venv/bin/python promover_cierre.py --aplicar --sin-confianza
# b) con el umbral calibrado (0,999): casi no sustituye; lo que sustituye es más fiable
ONEPEPE_DATA=~/.local/share/onepepe/cierre-trabajo ~/.local/share/onepepe/venv/bin/python promover_cierre.py --aplicar
```

Para volver atrás: los `.bak` con fecha de `~/.local/share/onepepe/math/` y `simbolos.anterior.npz`.

**3. Probar con el lápiz** la lista B1 de [`ARQUITECTURA-EJECUCION.md`](../planes/ARQUITECTURA-EJECUCION.md) §9 (12
pruebas) y el botón «Esto está mal…».

**4. Otro día: reentrenar** cuando termines los entrenos (115 de 400 escritos):
[`REENTRENAR.md`](../planes/REENTRENAR.md) trae el texto para pegarle a Claude.

**5. Para seguir mejorando**, por orden de efecto esperado:

- Modelo de pares para el corte (T5b): ataca los símbolos juntados y las rayas, que es lo que frena todo lo demás.
- Terminar los entrenos: el examen pasa de 22 a 80 expresiones y la red tiene más letra tuya.
- Una o dos páginas reales de apuntes con el modo matemáticas, para T9 (renglón vivo): el modo incremental pierde
  2 puntos en símbolos y 10 en renglones perfectos del examen.
