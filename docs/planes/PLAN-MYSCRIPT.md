# OnePepe: plan de implementación (funciones tipo MyScript Math)

Oct 29, 2026 · @Pepe

## Objetivo y orden

El plan añade a OnePepe seis bloques inspirados en MyScript Math, todo en local, sin tokens y sin IA en la nube. La regla ya existe, así que no entra.

Principios:

- Cada fase se puede usar sola y se prueba antes de pasar a la siguiente.
- Todo cálculo y toda comprobación van por reglas o por modelos pequeños entrenados con tu propia letra.
- Lo matemático vive en un módulo propio, para tocar lo mínimo de Rnote y poder actualizar el fork.
- Cada función se activa desde ajustes.

Orden de las fases, según tus prioridades:

1. Fase 1: tachar para borrar. Prioridad máxima.
2. Fase 2: diccionario de símbolos y catálogo. Prioridad alta.
3. Fase 3: variables. Prioridad muy alta.
4. Fase 4: revisión de álgebra paso a paso.
5. Fase 5: exportar a LaTeX.
6. Fase 6: fracciones y gráficos.

Las variables van antes que la revisión de álgebra y LaTeX porque solo necesitan un trozo pequeño de ellas: el árbol de expresión y un evaluador numérico básico. Ese mínimo se construye en la fase 3 y las fases 4 y 5 lo amplían después, sin rehacerlo.

## Fase 1: tachar para borrar

Tachar un trazo o un símbolo con un zigzag lo borra, sin cambiar de herramienta. Es la función más sencilla y la que más se nota al escribir.

1. Detectar el gesto: un trazo rápido que cambia de dirección al menos tres veces y es más ancho que alto, o al revés.
2. Elegir qué borrar: los trazos anteriores cuyo recuadro queda cubierto en su mayor parte por el zigzag. El zigzag mismo no se guarda.
3. Integrarlo en el historial para poder deshacer con un solo paso.
4. Ajustes: sensibilidad, y qué herramienta lo permite (bolígrafo, solo en modo matemáticas, o siempre).

Riesgo: confundir un tachado con una letra como la "w" o una "z" grande. Se reduce pidiendo que el gesto cubra trazos existentes y probando con tu letra.

## Fase 2: diccionario y catálogo de símbolos

MyScript reconoce un vocabulario cerrado. OnePepe define el suyo en un fichero de datos, que sirve a la vez de catálogo para insertar y de lista de candidatos para el reconocedor.

| Grupo | Contenido | Para qué |
| --- | --- | --- |
| Latinas y dígitos | a–z, A–Z, 0–9 | Base del reconocedor |
| Griegas | alfabeto completo | Biofísica y física |
| Operadores | + − × ÷ = ≠ ≤ ≥ √ Σ ∫ ∂ y similares | Álgebra y cálculo |
| Funciones | sin, cos, tan, ln, log, lim, det, exp... | Se reconocen como palabra, no como letras sueltas |
| Constantes | π, e, φ | Se interpretan como número |
| Unidades | SI con prefijos (k, m, µ, n...) | Biofísica |

- Cada símbolo lleva nombre, código LaTeX, categoría y alias para buscarlo.
- El catálogo es un panel de búsqueda que inserta el símbolo como texto o como trazo limpio.
- Se añaden aquí las constantes que uses en biofísica, en un fichero que puedes editar tú.
- Los elementos químicos y las unidades raras quedan como paquete opcional, activable desde ajustes.

## Fase 3: variables

Una línea como x = 3 define una variable, y las demás líneas que usan x se recalculan cuando cambia. Es la función de mayor prioridad después de tachar.

- **Árbol mínimo.** Una estructura que representa números, variables y operadores básicos, pensada para ampliarse en las fases 4 y 5.
- **Evaluador numérico.** Calcula el valor de una expresión sustituyendo las variables conocidas.
- **Definir.** Una línea de la forma letra = valor crea la variable, también con unidades (m = 70 kg). Una letra repetida es la misma variable por el contexto.
- **Usar.** Las líneas que contienen la variable muestran su resultado actualizado en un color suave a un lado, nunca encima de tu trazo.
- **Alcance.** La variable vale en toda la página; redefinirla actualiza todas sus líneas.
- **Ajustes.** Interruptor propio. El resultado con "=" sigue siendo una función aparte y opcional.

Riesgo: distinguir una x variable de un signo de multiplicación. Se resuelve con el contexto (una x entre números es producto) y se puede corregir a mano.

## Fase 4: revisión de álgebra paso a paso

Es tu idea de que se explique dónde está el fallo, sin IA ni tokens. La lógica es comprobar cada línea contra la anterior.

1. **Equivalencia.** Evaluar las dos líneas en muchos valores aleatorios de las variables. Si siempre coinciden, la línea es correcta; si no, hay un error. Es rápido y no necesita álgebra simbólica completa.
2. **Localizar.** La primera línea que deja de ser equivalente es donde está el fallo; se marca en rojo.
3. **Explicar.** Se prueban errores típicos sobre la línea anterior (signo perdido, no distribuir un factor, sumar fracciones numerador con numerador, cuadrado de una suma, pasar un término sin cambiar el signo). El que reproduce tu línea da la explicación.
4. **Sin explicación.** Si ninguna regla encaja, solo se dice "no es equivalente a la línea anterior".

Límites: no revisa demostraciones largas en prosa, y las ecuaciones donde se cambia de forma (por ejemplo, despejar) necesitan reglas propias. Se empieza por álgebra de una variable.

## Fase 5: exportar a LaTeX

Con el árbol de expresión de la fase 3, ampliado en la fase 4, el LaTeX sale de recorrerlo.

- Ampliar el árbol de la fase 3: fracciones, potencias, índices, raíces, funciones y unidades.
- Escribir el generador de LaTeX recorriendo el árbol, con una plantilla por tipo de nodo.
- Botón "Copiar como LaTeX" sobre una selección de trazos reconocidos.
- Pruebas: cada expresión de ejemplo debe dar el LaTeX esperado.

Se puede hacer en paralelo con la fase 4, porque ambas leen el mismo árbol.

## Fase 6: fracciones y gráficos

Son extras opcionales que aprovechan el árbol y el comprobador, y cada uno se activa por separado.

- **Fracciones.** Un botón junto a un resultado alterna entre decimal, fracción y fracción mixta.
- **Gráficos.** Al escribir y = f(x) se ofrece un recuadro con la curva sobre ejes cartesianos. Cambia al editar la ecuación. Enlaza con los ejes que ya querías de OneNote.
- **Aplazado.** Resolver desde una foto o PDF necesita reconocer matemáticas impresas; queda fuera de este plan.

## Riesgos y siguiente paso

El mayor riesgo es la ambigüedad al interpretar lo escrito, por ejemplo una x que es variable o un signo de multiplicación; afecta a las fases 3 a 5.

- Las fases 1 y 2 aportan valor por sí solas.
- Todo lo dicho sobre la estructura de Rnote debe verificarse leyendo el código de OnePepe.
- Las estimaciones de tiempo se dejan abiertas hasta saber cuánto código propio hay ya en el fork.

Siguiente paso: crear en GitHub una issue por fase con los criterios de cierre de este documento y empezar por la fase 1.

## Notas de Claude: qué existe ya en OnePepe (comprobado en el código, 2026-10-30)

- **Fase 1 (tachar):** no existe. La tinta a formas (`crates/rnote-compose/src/shaperecognition.rs`) sirve de modelo
  para detectar el gesto.
- **Fase 2 (diccionario):**
  - Ya existen:
    - el vocabulario `CORE` (`tools/onepepe-math/onepepe_math/symbols.py`);
    - los glifos limpios (`glyphs.py`);
    - las funciones `FUNCTIONS` (`layout.py`);
    - las unidades con pint;
    - el glosario por cuaderno (`glossary.py`);
    - el catálogo de símbolos para insertar.
  - Falta unirlo en un solo fichero de datos con nombre, LaTeX, categoría y alias.
- **Fase 3 (variables):** el árbol (`layout.py`) y el evaluador con SymPy (`evaluate.py`) ya existen. Se reutilizan,
  sin construir un árbol mínimo nuevo. Falta definir, recalcular y mostrar las variables.
- **Fase 4 (revisión):** ya existen los separadores de pasos `→ ⇒ ⇔ ;` (`STEP_SEPARATORS`).
- **Fase 5 (LaTeX):** `to_latex(tree)` ya genera el LaTeX de cada renglón. Falta el botón «Copiar como LaTeX».
- **Fase 6 (gráficos):** ya están planeados en `PLAN-PENDIENTE.md`, sección 2.

Seguimiento: `PLAN-PENDIENTE.md`, sección 7, y orden de prioridad.
