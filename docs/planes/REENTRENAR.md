# Reentrenar cuando termines los entrenos (para otro día)

Hoy (2026-11-15) hay 115 de 400 expresiones escritas. Cuando escribas las que faltan, el modelo tiene más letra tuya
de la que aprender y el examen pasa de 22 a 80 expresiones, así que las cifras serán más fiables. Este documento dice
qué hacer ese día.

## 1. Lo que haces tú antes

1. Escribe los entrenos que falten. Empieza por el set corto **«refuerzo»** (80 expresiones; Matemáticas → Nuevo
   entreno → refuerzo): está hecho con lo que más falla hoy (fracciones, números, símbolos de dos rayas). Después,
   los dos sets de 200 («General» y «Tus asignaturas»; Matemáticas → Abrir entreno).
2. No hace falta pulsar «Usar para entrenar» ni «Reentrenar»: el paso 2 lo reimporta todo desde los archivos
   `*.entreno.json` de `~/Documentos/OnePepe entrenos`.
3. Cierra OnePepe antes de empezar el paso 2.

## 2. El texto para pegarle a Claude

Abre una sesión nueva de Claude Code en `/home/pepe/Pepe_Flock/One_Pepe` y pega esto tal cual:

```text
Lee docs/planes/REENTRENAR.md y docs/estado/INFORME-CIERRE.md. He terminado de escribir más entrenos y quiero
reentrenar el modelo de matemáticas y saber si ha mejorado.

Haz esto, en la copia de trabajo (~/.local/share/onepepe/cierre-trabajo), sin tocar mis datos reales hasta el final:
1. Comprueba que mis datos reales no han cambiado desde el cierre (sha256sum -c origen.sha256). Si han cambiado,
   incorpora antes las correcciones nuevas a la copia de trabajo y dime cuántas son.
2. Cuenta cuántas expresiones hay escritas ahora en ~/Documentos/OnePepe entrenos y cuántas son del examen.
3. Guarda las medidas de hoy como «antes»: las de docs/estado/INFORME-CIERRE.md (archivos final-*.json en
   cierre-trabajo/medidas/).
4. El set «refuerzo» se añadió después de congelar el examen: comprueba que sus 15 ids de examen están en
   examen.json (si no, añádelos con la regla de M0.4 sin tocar los que ya hay) y que no entran en el entrenamiento.
   Después sigue los pasos de la sección 3 de REENTRENAR.md. Antes de ejecutar cada orden, comprueba en el código que hace lo
   que dice el documento (sobre todo cómo entran en la medida las expresiones del examen escritas después del cierre).
5. Compara antes y después con medir.py --comparar, en el examen y en la medida cruzada. Las expresiones nuevas del
   examen no existían en el «antes»: dame la comparación sobre las 22 expresiones antiguas y, aparte, las cifras del
   examen completo.
6. Instala el modelo nuevo solo si mejora o empata sin empeorar las sustituciones erróneas (regla M2.6 del manual).
   Antes de instalar nada en mis datos reales, enséñame la tabla y pregúntame.
7. Vuelve a calibrar el umbral de confianza y dime qué cobertura y qué error da.

Reglas: las de CLAUDE.md. Nunca elijas parámetros con el examen. Di qué está medido y qué no. Al final dame una tabla
«antes → después» de las seis metas y dime si ya merece la pena probar con el lápiz.
```

## 3. Los pasos, por si lo haces a mano

Todo se ejecuta en la copia de trabajo. Tus datos reales (`~/.local/share/onepepe/math`) no se tocan hasta el paso 7.

```bash
cd /home/pepe/Pepe_Flock/One_Pepe/tools/onepepe-math
PY=~/.local/share/onepepe/venv/bin/python
export ONEPEPE_DATA=~/.local/share/onepepe/cierre-trabajo
export ONEPEPE_MATHWRITING=~/.local/share/onepepe/mathwriting/mathwriting-2024
ENTRENOS="$HOME/Documentos/OnePepe entrenos"
```

| Paso | Orden | Qué hace | Tarda |
|------|-------|----------|-------|
| 1 | `$PY reimportar_entrenos.py --sesiones "$ENTRENOS" --aplicar` | Vuelve a importar toda tu letra con el alineado ordenado. Hace copia `.bak` antes. Lo del examen no entra | 1–2 min |
| 2 | `$PY train_classifier.py $ONEPEPE_MATHWRITING -o $ONEPEPE_DATA/nuevo-v3/simbolos.npz --personal $ONEPEPE_DATA/math/correcciones.jsonl --personal-weight auto --rasgos 2 --extraidos $ONEPEPE_DATA/mathwriting-extraido/simbolos.jsonl` | Entrena el modelo nuevo (prueba tres pesos de tu letra y elige) | 20–40 min |
| 3 | `$PY entrenar_pliegues.py --receta v3 --peso W --epocas N` | Los cuatro modelos de la medida cruzada. `W` y `N` son los que imprime el paso 2 al acabar | 30–40 min |
| 4 | `$PY medir.py --sesiones "$ENTRENOS" --conjunto examen --modelo $ONEPEPE_DATA/nuevo-v3/simbolos.npz --umbral 0 --salida $ONEPEPE_DATA/medidas/v3-examen.json` | Mide el modelo nuevo en el examen | 1 min |
| 5 | `$PY medir.py --sesiones "$ENTRENOS" --cruzada v3 --conjunto entreno --umbral 0 --salida $ONEPEPE_DATA/medidas/v3-cruzada.json` | Mide en letra no vista | 1 min |
| 6 | `$PY medir.py --comparar $ONEPEPE_DATA/medidas/final-cruzada-u0.json $ONEPEPE_DATA/medidas/v3-cruzada.json` (y lo mismo con `final-examen-u0.json` y `v3-examen.json`) | Dice si mejora (`P ≥ 0,80`), empata o empeora (`P ≤ 0,20`) | segundos |
| 7 | Solo si mejora: copiar `nuevo-v3/simbolos.npz` a `$ONEPEPE_DATA/math/simbolos.npz`, `$PY calibrar_umbral.py $ONEPEPE_DATA/medidas/v3-cruzada.json` y `$PY promover_cierre.py --aplicar` | Instala en tus datos reales, con copia `.bak` de cada archivo | segundos |

Avisos:

- Sin comprobar: las órdenes de los pasos 2 y 3 están escritas a partir de las que se ejecutaron en el cierre, pero
  esta tabla entera no se ha ejecutado de corrido. Por eso el texto del apartado 2 pide a Claude que las compruebe.
- Los nombres de los archivos de medida «antes» (`final-cruzada-u0.json`, `final-examen-u0.json`) son los del
  cierre; míralos con `ls $ONEPEPE_DATA/medidas/final-*`.
- La comparación de la medida cruzada no es del todo «igual con igual»: al escribir más expresiones cambian las que
  hay. `--comparar` solo usa las expresiones que están en las dos medidas.
- El equipo tiene 7 GB de memoria: un entrenamiento cada vez y sin la app abierta.
- Para volver atrás: los `.bak` con fecha están junto a cada archivo en `~/.local/share/onepepe/math/`; el modelo
  anterior es `simbolos.anterior.npz`.

## 4. Las cifras de hoy (el «antes»)

Medido el 2026-11-15 con el modelo v2, sin lápiz. Entre corchetes, el intervalo al 90 %.

| Medida | Examen (217 símbolos, 31 renglones) | Cruzada (813 símbolos, 108 renglones) | Meta |
|--------|------|------|------|
| Símbolos con corte y etiqueta | 78,3 % [73,4–82,6] | 70,2 % [67,5–72,8] | ≥ 90 % |
| Renglones todos bien | 54,8 % (17 de 31) | 38,0 % (41 de 108) | ≥ 60 % |
| Números de 2 o más cifras | 85,7 % (6 de 7) | 77,3 % (17 de 22) | ≥ 90 % |
| Raya de fracción, corte y papel | 16,7 % (1 de 6) | 10,7 % (3 de 28) | ≥ 90 % |
| Sustituciones erróneas (sin umbral) | 12,6 % (21 de 167) | 19,7 % (127 de 644) | ≤ 2 % |
| Resultados erróneos en las sondas | 0 de 24 | — | 0 |
| Símbolos juntados por error | 6,9 % | 13,4 % | ≤ 4 % |
| Red sola, primera candidata | 72,8 % | 65,9 % | ≥ 80 % |

Qué esperar: más letra tuya debería subir sobre todo «red sola» y «corte y etiqueta». Los símbolos juntados y las
rayas de fracción dependen del corte, no de la cantidad de letra: para eso está pendiente el modelo de pares (T5b,
`docs/estado/INFORME-CIERRE.md`).
