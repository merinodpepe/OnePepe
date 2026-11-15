# Auditoría del modo matemáticas (2026-11-15)

Esta auditoría explica por qué el modo matemáticas falla y con qué datos. Lo que hay que hacer, en orden, está en
[`PLAN-CIERRE.md`](PLAN-CIERRE.md).

> **Nota (2026-11-15).** Medido y escrito sobre la versión 1.3.5 que había en el equipo. El remoto ya iba por la
> 1.4 (el punto multiplica y la coma es el decimal), que en matemáticas solo cambia `evaluate._tokens`,
> `layout._NEVER_SCRIPT` y que los resultados salen con coma. Las medidas de corte y etiqueta no dependen de eso.
> Las 8 sondas con resultado erróneo se repitieron sobre la 1.4 y siguen igual (la de unidades da ahora
> «686,0 g⋅km/s²»). Sobre la 1.4 pasan 88 tests.

## En dos minutos

1. **El problema nº 1 no es que las cifras se junten. Es la etiqueta con letra no vista.** Con el corte bien hecho,
   la red sola acierta el símbolo el 86,1 % de las veces con lo que ya ha visto (medido). Con expresiones tuyas que
   no ha visto, baja a 53,8–59,1 % en top-1 y 76,0–83,5 % en top-5 (medido). Renglones enteros bien con letra no
   vista: 5,0–27,6 % (medido).

2. **Un número de dos cifras o un «dx/dt» falla porque los errores se multiplican.** Nada trata el conjunto como
   una unidad. Si cada símbolo acierta ~80 %, dos cifras aciertan ~64 %, y cinco símbolos (d, x, raya, d, t) ~33 %.
   Con letra no vista (~58 %), dos cifras aciertan ~34 %. Medido: una cifra dentro de un número acierta 86,1 %,
   frente al 79,5 % de una cifra aislada (F). El número no es más difícil por símbolo; es más largo.

3. **La raya de fracción es el peor símbolo.** Corte exacto: 52,6 %. Corte más etiqueta: 21,1 % (n=38). En el
   44,7 % se junta con un vecino (medido). Por eso fracciones y derivadas casi nunca salen.

4. **Los ejemplos de entrenamiento propios están contaminados.** `forced_groups` ignora el orden de los símbolos.
   En 43 de 154 renglones (28 %) da grupos distintos del alineado ordenado. Ejemplo: en «2x+1=7», «=» queda con 1
   trazo y «7» con 3 (medido). `import_session` lo usa, así que parte de los 1896+1066 ejemplos importados está mal
   cortada o mal etiquetada, y se reentrena con ellos.

5. **El motor simbólico muestra resultados erróneos.** En 8 casos sondeados (sonda): `f(3)=` da «3·f»;
   `f(x)=` con x=5 da «5·f»; `dx/dt` da «x/t»; `dy/dx` da «y/x»; `d²/dx²(x³)` da «3x²»; `dx/dt` en línea da «t·x»;
   `(x+1)+3=` da «3x+3»; `m=70 kg`, `g=9,8 m/s²`, `mg=` da «686 g·km/s²». Con «Variables» activado se rompen
   integrales, derivadas, sumatorios y la revisión de pasos (sonda).

6. **Las opciones «=», «Variables» y «Revisar pasos» vienen apagadas por defecto** (código: `MathOptions`:
   `equals=false`, `variables=false`, `check=false`). Con «Variables» apagado, `x=5` / `2x+3=` da «?» (sonda).
   Parte de «no resuelve la variable» puede ser solo eso.

7. **Una de cada diez sustituciones «seguras» es errónea.** Con confianza ≥ 0,95, son erróneas el 9,0 % (F, 66 de 730) y el
   10,5 % (O, 61 de 583) (medido). El umbral no está calibrado.

## Cómo se hizo y qué validez tiene

- Auditoría de solo lectura. No se cambió código, ni el formato .rnote, ni los datos del usuario, ni git.
- Se leyeron enteros (código): `tools/onepepe-math/onepepe_math/` (`segment`, `features`, `classifier`, `context`,
  `recognizer`, `dots`, `layout`, `evaluate`, `pagina`, `forced`, `personal`, `symbols`), `train_classifier.py`,
  `reentrenar.py`, y de Rust `crates/rnote-engine/src/engine/math.rs` (cola, petición `recognize`, respuesta,
  renglones, petición `page`).
- Dos agentes Sonnet midieron: (a) corte y reconocimiento con las sesiones de entreno del usuario; (b) 28 sondas
  del motor de evaluación. Scripts y resultados (sin trazos) están en
  `~/.local/share/onepepe/auditoria-2026-11-15/{medidas,sondas}/`, fuera del repositorio y nunca en git.
- Datos usados (medido): 115 expresiones escritas (85 del set «general», 30 de «asignaturas»), 157 renglones y
  1140 símbolos. Ojo: son 115, no 400. Los dos sets de 200 están a medias.
- **Aviso de validez.** El modelo actual se reentrenó el 2026-11-11 con 3130 ejemplos, y entrenó con esas mismas
  expresiones. Por eso las medidas «visto» son optimistas. La medida honesta es la «no visto»: entrenar con un set
  y probar con el otro, sin kNN.
- **Dos referencias.** F es el corte forzado actual (`forced_groups`), con 154 renglones. O es el alineado ordenado,
  que respeta el orden del LaTeX, solo en renglones sin `\frac`: 127 renglones y 844 símbolos. O es la más fiable.
- Nada de esto está probado con el lápiz en la app. Son medidas fuera de la app sobre lo escrito en el modo entreno
  (caja guiada). En uso real, las pausas y el tamaño serán distintos.
- Batería existente (medido): 85 tests de Python, 0 fallos, 1 saltado.

### Reservas

- Nada con lápiz en la app. Las pausas y tamaños son de la caja de entreno, no de apuntes reales.
- Trazos no consecutivos: solo hay cota superior. `\frac` frente a «−» a través de la estructura: no medido. La
  estructura trata igual las dos etiquetas si la raya es larga y tiene algo encima y debajo (código).
- Hay muestras pequeñas en varias filas (30 expresiones en «asignaturas», 20–50 pares de cifras): mucha
  incertidumbre.
- La referencia F se calculó con el modelo actual, así que es circular. Por eso O es preferible.
- Sondas de lógica: la geometría es una estimación, y los umbrales valen solo para ella.
- Causas sin verificar: `x+y=` con x definida; la unidad «g·km/s²».
- Las metas y las cifras de aceptación son propuestas del arquitecto, no promesas.

## Eje 1: agrupar trazos y símbolos compuestos

Es la prioridad máxima del usuario. La hipótesis de que las cifras se juntan quedó refutada (ver más abajo). La
explicación real es que los errores se multiplican y que ninguna capa trata un número o una derivada como una
unidad.

### Cómo funciona hoy (código)

- App: tras 0,8 s sin escribir (`RECOGNITION_DELAY`, `mathhelper.rs`), envía solo los trazos nuevos
  (`math_take_request`, `math.rs:548`).
- Los símbolos ya reconocidos del renglón van como `known`: fijos, con probabilidad 1. No se vuelven a cortar ni a
  etiquetar.
- `MathSymbol` guarda `original` (los trazos a mano), así que reabrirlos es posible sin cambiar el .rnote.
- `segment.py`: solo forma grupos de 1–4 trazos consecutivos en el tiempo. Usa programación dinámica con puntuación
  `log p_top + log(1−p_trozo) − 1,2` por símbolo, más pausas.
- `_close` decide qué trazos pueden ir juntos. Pueden ir juntos los «apilados» (solape horizontal >
  mín(0,3·estrecho, 0,2·típico) − 0,35·típico, y hueco vertical < 1,0·típico) o los que «se tocan» (huecos
  < 0,25·típico).
- `dots.py`: es la única reparación posterior (punto de i/j).
- No existe ningún nivel «léxico». Las cifras se unen en número al final (`evaluate._tokens`), sin límite de hueco.
  `dx`, `d/dx` y `dx/dt` no existen como entidad.

### Medidas (medido)

Corte de todos los trazos de una vez. F es el corte forzado actual; O es el alineado ordenado. Cifras en %.

| Resultado            | F (n=1140) | O (n=844) |
|----------------------|-----------:|----------:|
| Exacto               | 85,2       | 90,0      |
| Juntado              | 8,9        | 5,0       |
| Partido              | 2,5        | 2,8       |
| Mixto                | 3,3        | 2,1       |

- Con letra no vista, corte exacto: 70,0–82,4 % (F) y 73,3–91,4 % (O).

Por tipo de grupo (F):

| Grupo          | Resultado                                  |
|----------------|--------------------------------------------|
| 1 trazo        | 87,2 % exacto; 12,8 % juntado              |
| 2 o más trazos | 81,7 % exacto; 7,0 % partido; 9,1 % mixto  |

Por símbolo (corte exacto, F):

| Símbolo              | Exacto (%) | n   | Nota                                      |
|----------------------|-----------:|----:|-------------------------------------------|
| «=»                  | 83,9       | 112 | En O absorbe a un vecino en 7 de 42       |
| «+»                  | 76,9       | 52  |                                           |
| «x»                  | 87,2       | 86  |                                           |
| «t»                  | 81,8       | 11  |                                           |
| «7»                  | 81,2       | 16  |                                           |
| «1»                  | 50         | 14  |                                           |
| «H»                  | 0          | 4   |                                           |
| «π»                  | 0          | 2   |                                           |
| Raya de fracción     | 52,6       | 38  |                                           |
| «d» de diferencial   | 81,8       | 11  |                                           |

Números:

| Caso                                                | F              | O              |
|-----------------------------------------------------|---------------:|---------------:|
| Número de 2 o más cifras, todo bien (corte y etiqueta) | 77,4 % (n=31) | 82,6 % (n=23) |
| Cifras aisladas bien                                | 79,5 % (n=259) | 81,1 % (n=206) |

- Con letra no vista, las cifras dentro de un número salen bien en 63,3–78,3 % (n=49 y 23).

### Hipótesis refutada: «la penalización junta cifras vecinas» (medido)

- De 61 pares de cifras vecinas, «juntas» gana 1 vez.
- `_close` solo permite unirlas en el 15–20 % de los casos.
- La clase «trozo» da p≈0,99–1,0 a la unión.
- Barrido de `symbol_penalty` de 1,2 a 0: corte exacto de 85,2 a 87,2 (F); exacto más etiqueta de 75,4 a 77,4;
  partidos casi igual (2,5 a 2,7). La mejora es pequeña, pero gratis.

### Otras medidas

- Trazos no consecutivos (la raya de la t puesta después, etc.): no hay medida directa. La cota superior es ~10 %
  de los renglones (13 de 127 mejoran al mover un trazo en símbolos multitrazo: `=`, `x`, `+`, `b`, `7`, `d`, `k`,
  `q`).
- Lotes por pausa (simulación de la app, medido):

| Pausa de corte | Lotes por renglón | Símbolos multitrazo repartidos en lotes distintos | Renglones con alguno |
|---------------:|------------------:|--------------------------------------------------:|---------------------:|
| 0,8 s          | 1,66              | 3,6 % (n=416)                                     | 9,1 %                |
| 1,5 s          | —                 | 1,9 %                                             | 5,2 %                |
| 2,5 s          | —                 | 1,4 %                                             | 3,9 %                |

- El coste de los lotes es de ~1 punto (75,4 a 74,6 en exacto más etiqueta).
- Pausas dentro de un símbolo: mediana (p50) 0,13 s, p90 0,32 s, y el 3,4 % supera 0,8 s (medido).
- Es la caja del entreno. En apuntes reales la pausa será peor (hipótesis).
- Estructura (sonda): la segunda cifra de un número pasa a subíndice si mide ≤ 0,72 de la primera, con la base
  alineada. Pasa a exponente con 0,6 y subida: «3⁰». Así, `x+3⁰` se evalúa como x+1, que es un resultado erróneo.
  Al 80 %, depende del desplazamiento: subíndice con +0,1 del alto hacia abajo; exponente con −0,3 o más.
- Estructura (sonda): no hay límite de hueco. Dos números separados 3 veces el alto se funden en uno.
- Estructura (sonda): un «=» partido en dos rayas, un «+» partido, un «≤» partido en «<» y «−», un «÷» y una «i» partidos no
  se recomponen. Salen «2+3--», «2-|3=», «x<-3» y «6-..3=».

### Causas, por orden de peso (código)

- **A. No hay entidades.** Los errores por símbolo se multiplican y ninguna capa usa «esto es un número» o «esto es
  una derivada» para corregir. Las confusiones medidas con letra no vista son: b→D 15, 5→S 9, 9→q 9, x→X 7,
  g→9 6, n→\hat 6, s→S 5 (medido).
- **B. La regla «apilados» de `_close` es demasiado amplia.** Una raya larga se solapa en horizontal con todo lo que
  tiene encima y debajo. Así, la raya de fracción y el «=» se tragan al vecino.
- **C. Solo trazos consecutivos y máximo 4.** La única reparación es la de i/j.
- **D. Compromiso irrevocable tras 0,8 s.** Lo ya reconocido no se reabre.
- **E. La estructura decide potencia o subíndice comparando con el símbolo anterior,** no con la línea base del
  renglón. Las cifras no tienen trato especial.
- **F. Las derivadas solo existen como `d/dx` seguido de algo.** `evaluate.atom` lo reconoce así. `dx/dt` se lee
  como `(d·x)/(d·t)`.

### Propuesta: cuatro niveles en vez de dos

Nivel 0 trazos → Nivel 1 símbolos → **Nivel 2 léxico (nuevo)** → Nivel 3 estructura → Nivel 4 significado.

- **N1a «Rayas primero».** Un trazo largo y plano (ancho > 1,5·típico y alto < 0,25·ancho) no se une por la regla
  de apilar, salvo con otro trazo plano de ancho parecido (`=`, `≡`, `≤`, `≥`, `≠`), con puntos pequeños (`÷`) o
  con «+» (`±`). Hay que estrechar el solape de «apilados» (quitar el margen de −0,35·típico) y bajar
  `symbol_penalty` a 0–0,3, elegido en el banco de medida.
- **N1b «Modelo de pares».** Sustituye a la penalización fija. Es una regresión logística de
  P(mismo símbolo | hueco x, hueco y, solape, razón de tamaños, planos o no, pausa, nº de trazos). Se entrena con el
  alineado ordenado de MathWriting y de tu letra. Entra en la DP como log-probabilidad de unir o cortar entre
  trazos vecinos.
- **N1c «Reparación general».** Va después de la DP, en `dots.py`, que pasaría a `reparar.py`. Para cada símbolo
  sobrante típico (punto, rayita, «−», «|», «)», trozo con p_trozo alta), prueba a unirlo con el símbolo que
  solapa en el espacio, aunque no sea consecutivo en el tiempo. Se acepta si la puntuación total mejora. Cubre
  `=`, `≤`, `≥`, `≠`, `÷`, `±`, i, j, `!`, `:`, `;`, y las rayas tardías de t, 7, H, x, +.
- **N1d «Tamaño relativo» como rasgo.** Hoy los rasgos no dependen del tamaño, porque `features.py` normaliza. Hay
  que añadir el ancho y alto del grupo respecto al tamaño típico del renglón, y su posición respecto a la línea
  base. Así se distingue x/X, s/S, c/C, o/0, «.»/«·», «,»/«'», «−»/raya de fracción, y se detectan dos símbolos
  juntados.
- **N2 «Léxico»** (módulo nuevo `lexico.py`, entre `layout` y `evaluate`):
  - Número: cifras vecinas con la base alineada (fondos a ±0,2 del alto) son un número, aunque la segunda mida
    70–85 %. Hueco máximo ~0,8·típico. Coma o punto decimal.
  - Corrección dentro de una entidad: en una tirada de cifras, un parecido con una cifra entre sus alternativas se
    lee como cifra (S→5, O/o→0, l/|/I→1, g/q→9, b→6, Z/z→2, B→8). En un nombre de función ya se hace («1n»→ln).
  - Diferencial «d» más variable. Operador `d/dx`. Derivada de Leibniz `dy/dx`, `d²y/dx²`, `∂f/∂x`. Forma en
    línea `dx/dt`. Prima `f'(x)`. Unidades (kg no es k·g).
  - Salida: nodos `("num")`, `("diff")`, `("deriv", función, variable, orden)`, `("call", f, args)`, para que
    `evaluate` no tenga que adivinar.
- **N3 «Estructura con línea base del renglón».** Ajustar una recta a los fondos de los símbolos sin rabo, con
  regresión robusta. Admite renglón inclinado (letra «poco lineal»). Con esa recta se decide potencia o subíndice.
  Un operador (`+`, `−`, `=`) tras `)` nunca es subíndice. Los límites de `∑`, `∏` y `lim` se asignan por geometría
  (debajo o encima), y pueden contener `=` y `→`.
- **«Renglón vivo»** (Rust más Python), para la causa D. La petición `recognize` lleva también los trazos originales
  de los símbolos vecinos aún no fijados. Son los que tocan, o están a < 0,6·típico de un trazo nuevo, o son la cola
  del renglón. El programa auxiliar recorta el conjunto y la app aplica la diferencia: los que no cambian conservan
  su id. La sustitución por el símbolo limpio se retrasa en la cola hasta que hay un símbolo más a su derecha o
  pasan ~2,5 s. Lo que corriges a mano queda bloqueado.
  - Antes de construirlo, hay que medir las pausas reales con los `times` de los archivos ocultos de páginas
    reales. Solo se hace si más del 3 % de los símbolos multitrazo quedan partidos.
  - Mitigación barata e inmediata: retardo configurable, de 0,8 a 1,2 s.

## Eje 2: variables y contexto

### Cómo funciona hoy (código)

- No hay estado guardado. La app manda todos los renglones de la página (`math_page_request`) tras cada cambio, y
  `pagina.page` recalcula todo. Es un diseño sano, porque no hay estado que se desincronice, y es rápido: 40
  renglones con todo activado tardan 0,140 s; 24 ms con listas repetidas; el peor renglón, 164 ms (sonda).
- Definición es solo «letra = número» (o número con unidad).
- `_env_for`: vale la última definición de encima. Si no hay ninguna encima, vale la primera de la página.
- La sustitución se hace al analizar (`evaluate._Parser._symbol` devuelve el valor del glosario o del entorno). Así
  el árbol nunca ve la variable.

### Qué funciona (sonda, con «Variables» activado)

- `x=5` / `2x+3=` → 13.
- `y=2x+3` / `y=` → 13.
- Cadenas: `a=2`, `b=3`, `c=a+b`, `c²=` → 25.
- Redefinir x más abajo.
- Subíndices `x_1`, griegas, `μ=0,5` (coma), `ŷ`.
- Mayúscula distinta de minúscula.
- `x=12` / `x+30=` → 42.
- `12+34=`, `3,5+1,5=`, `1000/8=`, «2 x 3» → 6.
- `d/dx(x²)=` → 2x. `∂/∂x(x²y)=` → 2xy. `∫x dx` sin definida.

### Qué falla (sonda)

Todo lo de esta tabla es sonda. Las causas que no llevan «sin verificar» se leyeron en el código.

| Escribes | Qué sale | Causa |
|----------|----------|-------|
| Con «Variables» apagado: `x=5` / `2x+3=` | «?» | Opciones apagadas por defecto: `MathOptions::default` (`math.rs:49`) |
| `x=5` / `y=2x+3` | No muestra nada | `pagina.page`: los renglones de definición no llevan marca |
| `f(x)=x²+1` / `f(3)=` | «3·f» **resultado erróneo mostrado** | `evaluate._Parser` no conoce funciones del usuario: `f(` es un producto |
| `f(x)=x²+1` / `f(x)=` con x=5 | «5·f» **resultado erróneo mostrado** | Igual que la fila anterior |
| `f(x)=` sin x | «f·x» | Igual que la fila anterior |
| `x=t²` / `dx/dt` | «x/t» **resultado erróneo mostrado** | `evaluate.atom` solo reconoce `frac(d, d·var)` seguido del operando |
| `y=x²` / `dy/dx` | «y/x» **resultado erróneo mostrado** | Igual que la fila anterior |
| `dx/dt` en línea | «t·x» **resultado erróneo mostrado** | Igual que la fila anterior |
| `d²/dx²(x³)=` | «3x²» **resultado erróneo mostrado** (depende de la geometría de la sonda) | `layout._parse_region`: el filtro de 1,5·típico deja fuera el exponente del numerador |
| `f'(x)=` | «?» | La prima se lee como exponente |
| `x=5` / `∫2x dx=` | «?» | Se sustituye x=5 dentro de la variable ligada |
| `x=5` / `∫₀¹x²dx=` | «?» | Igual que la fila anterior |
| `x=5` / `d/dx(x²)=` | «?» (sin «Variables» sale bien) | Igual que la fila anterior |
| `∑_{i=1}^{3} i =` | «?» SIEMPRE, también sin variables | `layout`: «=» está en `_NEVER_SCRIPT`, y el límite «i=1» no puede ser subíndice. Los tests no lo ven porque construyen el árbol a mano |
| `(x+1)+3=` | «3x+3» **resultado erróneo mostrado** (debía ser x+4) | `layout._parse_row`: tras `)`, un «+» algo bajo se toma por subíndice. Los símbolos planos cuentan siempre como «más pequeños» |
| `x=5` / `x+y=` | «?» (se esperaba y+5) | Causa sin verificar |
| `e^5=` | «?» | «e» no es la constante; solo «pi» lo es |
| `m=70 kg` / `g=9,8 m/s²` / `mg=` | «686.0 g·km/s²» **resultado erróneo mostrado** (en la unidad) | `kg` se lee como k·g. Causa exacta sin verificar |
| «Revisar pasos» + «Variables»: `2x+4=10` / `2x=14` / `x=7` / `x=3` | Ningún error marcado (sin «Variables» sí) | `_env_for` mete «x=7» de más abajo dentro de «2x=14» y la da por buena |

### Propuesta: módulo `entorno.py` (tabla de símbolos de la página)

- Tipos de definición: valor (`x=5`), cantidad con unidad, EXPRESIÓN (`y=x²`, alias simbólico) y FUNCIÓN
  (`f(x)=x²+1`, con parámetros).
- Analizar sin sustituir: siempre símbolos. Calcular las variables ligadas (variable de integración, de `d/dx`,
  índice de `∑` o `∏`, variable del límite, parámetros de función). Sustituir solo las libres con `expr.subs`, en
  orden de dependencias.
- Grafo de dependencias entre renglones: cambiar x recalcula solo lo que depende de x. Detectar ciclos
  (`x=x+1` no es definición).
- Llamadas: «letra(» es llamada si esa letra está definida como función en la página. Si no lo está y la letra es
  f, g, h, F o G, sale «?» en vez de producto.
- Derivadas con alias: `dy/dx` con `y=x²` definida da 2x; `dx/dt` con `x=t²` da 2t; también `f'(x)`.
- Los renglones de definición enseñan su valor cuando el lado derecho usa otras variables. Entorno parcial:
  `x+y=` con x=5 da `y+5`.
- Revisión de pasos: dentro de un desarrollo no se sustituye la variable que se está despejando.
- «e» es la constante de Euler, salvo que la página o el glosario la definan.
- Persistencia: no hace falta guardar nada, porque se recalcula desde la página. Caché por renglón, con clave igual al
  contenido del renglón más el entorno que usa, para no reevaluar todo.
- Decisión del usuario pendiente: encender por defecto «=» y «Variables» cuando esto esté arreglado. Hoy están
  apagadas.

## Eje 3: precisión general y tu dataset

### Medidas (medido)

Red sola con el corte bueno y letra no vista. Top-1 y top-5 en %.

| Grupo                          | Top-1          | Top-5          |
|--------------------------------|---------------:|---------------:|
| Total A → asignaturas (n=413)  | 53,8           | 76,0           |
| Total B → general (n=727)      | 59,1           | 83,5           |
| Cifras                         | 57,0–58,3      | 75,9–84,9      |
| Latinas                        | 58,9–60,7      | 76,2–83,7      |
| Operadores                     | 40,0–42,4      | 70,9–74,6      |
| Relaciones                     | 73,2–85,3      | 97,3–100       |
| Griegas (n=25)                 | 40,0           | 52,0           |

- Modelo actual sobre lo visto: top-1 86,1 %, top-5 96,0 %.
- Con MathWriting (símbolos reservados): top-1 0,595–0,631, top-5 0,895–0,907.

Pipeline completo, en %:

| Medida                         | Visto F | Visto O | No visto F    | No visto O    |
|--------------------------------|--------:|--------:|--------------:|--------------:|
| Símbolos con corte y etiqueta  | 75,4    | 79,0    | 44,1–58,5     | 51,3–65,6     |
| Renglones todos bien           | 44,2    | 53,5    | 5,0–23,7      | 6,9–27,6      |
| Renglones con corte perfecto   | 62,3    | 70,9    | —             | —             |
| LaTeX exacto                   | 40,3    | 48,0    | —             | —             |

- Reparto del error (visto, F): el corte falla en 169 de 1140 (14,8 %). La etiqueta, con corte bueno, falla en 111
  de 971 (11,4 %).
- Confusiones con corte bueno (visto): `\frac`→− 7; −→`\frac` 6; `\overline`→− 6; «,»→1 5; 1→) 4; =→− 3.
- Confianza (visto): los aciertos tienen mediana (p50) 0,9999. Los fallos tienen p50 0,63, pero 18 de 111 fallos
  están en ≥ 0,95.

### Causas (código)

1. Pocos datos genéricos. El clasificador se entrena con los 6 423 símbolos sueltos de MathWriting (80 % ≈ 5 100; ~22
   por clase en 229 clases). Las ~230 000 fórmulas completas de `train/` solo se usan para el modelo de pares de
   símbolos.
2. Ejemplos propios mal alineados (ver punto 4 del resumen): `forced_groups` no tiene en cuenta el orden.
3. Fuga en la validación. `train_classifier.py` deforma las muestras (×30; las propias, ×90) antes de que
   `MLPClassifier(early_stopping=True)` separe su 10 % de validación al azar. Las copias deformadas de una misma
   muestra caen en los dos lados. Así, la parada temprana no mide nada real.
4. Peso de lo propio sin control. Son ~1 283 símbolos propios × 91 ≈ 117 000 muestras, frente a ≈ 5 100 × 31 ≈
   158 000 genéricas (más los «trozos»). La red memoriza las expresiones ya escritas: 86 % visto frente a 54–59 % no
   visto.
5. Rasgos pobres (`features.py`): 32 puntos para todo el símbolo, que dependen del orden y sentido de los trazos.
   Rejilla binaria 10×10. Sin tamaño relativo ni posición en el renglón.
6. Sin conjunto de examen fijo. `reentrenar.py` reserva el 30 % más reciente por ejemplo (no por expresión) y
   compara red más kNN. Esos mismos ejemplos entran en el siguiente reentrenamiento. No hay forma de saber si se
   mejora de verdad.
7. Confianza sin calibrar: entre el 9 % y el 10,5 % de las sustituciones «seguras» son erróneas.
8. El modelo de contexto (bigramas) lee el renglón de izquierda a derecha por el centro de las cajas. En fracciones,
   potencias y límites mezcla numerador y denominador.

### Cómo integrar tu dataset nuevo sin sobreajustar (receta)

1. Catálogo: cada muestra con origen, sesión, fecha, id de EXPRESIÓN y tipo (entreno, página etiquetada, corrección
   en la app). Quitar duplicados.
2. Alinear con el alineado ORDENADO, no con `forced_groups`. Para fracciones, raíces y acentos, probar los órdenes
   de escritura posibles: raya antes, en medio o después.
3. Control de calidad: una muestra entra solo si su etiqueta está entre las 5 primeras de la red, o coincide con el
   vecino kNN. Lo dudoso va a una cola de revisión, no al entrenamiento.
4. Reparto POR EXPRESIÓN, nunca por símbolo: examen del 15–20 % congelado para siempre (no entra en entrenamiento ni
   en el kNN de las medidas); validación del 15 % (parada temprana, peso de lo propio, calibración); el resto,
   entrenamiento. Estratificado por set y por tipo.
5. Deformar DESPUÉS de repartir. Parada temprana con la validación real.
6. Mezcla equilibrada por clase: tope de muestras por clase y origen. El peso de lo propio (p. ej. ×10, ×30, ×90) se
   elige con la validación, no a ojo.
7. Aceptación del modelo nuevo: mejor en el examen propio, y no más de 2 puntos peor en los símbolos reservados de
   MathWriting, y no más sustituciones erróneas.
8. Curva de aprendizaje: entrenar con 25, 50, 75 y 100 % de lo propio. Si entre 75 y 100 % ya no sube, escribir más
   no ayuda y hay que cambiar de modelo.
9. Más datos genéricos antes que más propios. Sacar símbolos de las fórmulas `train/` de MathWriting con el alineado
   ordenado y un filtro de confianza (dos rondas). O usar CROHME, que trae el corte en símbolos hecho; comprobar
   licencia y descarga; nunca a git.
10. Calibrar la confianza (temperatura) con la validación, y guardarla en el .npz.

### Preguntas sobre el dataset (para ti)

- Formato: `.entreno.json`, páginas .rnote con su texto, u otra cosa.
- Cuántas expresiones.
- Si lleva tiempos por trazo.
- Si está escrito con el tamaño y zoom habituales.

## Eje 4: cómo razona el sistema

Cuellos de botella estructurales (código):

1. **Tubería voraz.** Cada etapa se queda con UNA respuesta, y la siguiente no puede discutirla: corte, etiqueta,
   estructura, cálculo. Las alternativas solo se usan para unir nombres de función. La gramática nunca corrige al
   reconocedor.
2. **Lo ya reconocido es intocable.** Son `known` con p=1. Un error temprano contamina el contexto de lo siguiente.
3. **El contexto mira una secuencia plana, no la estructura** (causa 8 de la sección de Eje 3).
4. **Dos cortes en renglones distintos.** Rust (`math_lines_of`, solape vertical con margen 0,3) y Python (`layout`).
   Ninguno ve lo que decide el otro.
5. **La sustitución de variables al analizar destruye información** (ver Eje 2).
6. **La incertidumbre no llega al resultado.** Un renglón con símbolos dudosos da un resultado con el mismo aspecto que
   uno seguro. A veces un resultado erróneo es peor que ninguno.
7. **Los errores se tragan.** `except Exception: pass` en `pagina.py` y `evaluate.py`. Cuando no sale nada, no queda
   rastro de por qué.
8. **No hay circuito de fallos reales.** Lo que falla en clase no se guarda como caso de prueba ni como dato.
9. **Se recalcula la página entera en cada cambio.** Hoy es barato (0,14 s con 40 renglones). Hay que vigilarlo con
   SymPy, que es pesado.

Propuestas:

- **a)** N mejores más gramática: probar alternativas en los símbolos de menor margen (haz ≤ 32), y preferir la
  lectura que se puede analizar (paréntesis cerrados, operador entre operandos, función conocida).
- **b)** Contexto en el orden del árbol.
- **c)** Un solo dueño del corte en renglones: el programa auxiliar.
- **d)** Confianza del renglón igual al mínimo de sus símbolos, y resultado «dudoso» visible.
- **e)** Traza por renglón («qué leí y por qué no calculo») en un registro local.
- **f)** Botón «Esto está mal», que guarda los trazos y lo que debía ser, en `fallos.jsonl` local. Así entra al banco
  de medida y al entrenamiento.
